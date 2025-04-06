use crate::constants::IGNORE_LIST;
use crate::error::PDFConError;
use crate::pdf_image::PDFConColorSpace;
use crate::pdf_image::optimize::optimize_jpeg_mem;
use crate::{Run, pack::ImageType};
use indicatif::ParallelProgressIterator;
use log::{debug, error};
use lopdf::{Dictionary, Document, Object, ObjectId};
use rayon::prelude::*;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpack {
    pub threads: usize,
    pub out_directory: PathBuf,
    pub in_file: PathBuf,
    pub optimize: bool,
}

pub fn filter_func(object_id: (u32, u16), object: &mut Object) -> Option<((u32, u16), Object)> {
    if IGNORE_LIST.contains(&object.type_name().unwrap_or_default()) {
        return None;
    }

    if let Ok(d) = object.as_dict_mut() {
        d.remove(b"Produce");
        d.remove(b"ModDate");
        d.remove(b"Creator");
        d.remove(b"ProcSet");
        d.remove(b"Procset");
        d.remove(b"MediaBox");
        d.remove(b"Annots");
        if d.is_empty() {
            return None;
        }
    }

    Some((object_id, object.to_owned()))
}

impl Unpack {
    pub fn process_jpg(&self, page_num: u32, content: &Vec<u8>) -> Result<(), PDFConError> {
        // Save image
        if self.optimize {
            let file_name = format!("{:0>5}.jpg", page_num);
            debug!("Saving: {}", file_name);
            let path = self.out_directory.join(file_name);
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&optimize_jpeg_mem(content)?)?;
            writer.flush()?;
            optimize_jpeg_mem(content)?;
        } else {
            let file_name = format!("{:0>5}.jpg", page_num);
            debug!("Saving: {}", file_name);
            let path = self.out_directory.join(file_name);
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&content)?;
            writer.flush()?;
        }

        Ok(())
    }

    pub fn process_xobject(&self, doc: &Document, page_num: u32, reference: &Object) {
        let ref_id = match reference.as_reference() {
            Ok(d) => d,
            Err(_e) => {
                error!("Failed to get references id");
                return;
            }
        };

        let xobj = match doc.get_object(ref_id) {
            Ok(d) => d,
            Err(_e) => {
                error!("Failed to get the xobject");
                return;
            }
        };

        let stream = match xobj.as_stream() {
            Ok(d) => d,
            Err(_e) => {
                error!("Failed to get stream from xobject");
                return;
            }
        };

        let subtype = match stream.dict.get(b"Subtype") {
            Ok(s) => s,
            Err(_e) => {
                error!("Failed to get stream subtype");
                return;
            }
        };

        match subtype.as_name() {
            Ok(name) => {
                if name != b"Image" {
                    debug!("subtype is not an image");
                    return;
                }
            }
            Err(_e) => {
                error!("Failed to get subtype as name");
                return;
            }
        }

        let filter = match stream.dict.get(b"Filter") {
            Ok(d) => d,
            Err(_e) => {
                error!("Failed to get stream filter");
                return;
            }
        };

        let filter_name = match filter.as_name() {
            Ok(d) => d,
            Err(_e) => {
                error!("Failed to convert to filter name");
                return;
            }
        };

        if filter_name == b"DCTDecode".to_vec() {
            // Process JPEG image
            match self.process_jpg(page_num, &stream.content) {
                Ok(()) => {}
                Err(e) => {
                    error!("Failed to write image contents to file: {}", e.to_string());
                }
            }
            return;
        }

        if filter_name == b"FlateDecode".to_vec() {
            // Process PNG image
            // We need more information to re-encode this. Grab the width, height and color_type
            let width_obj = match stream.dict.get(b"Width") {
                Ok(w) => w,
                Err(_e) => {
                    error!("Failed to get png width object");
                    return;
                }
            };

            let width = match width_obj.as_i64() {
                Ok(i) => i,
                Err(_e) => {
                    error!("Failed to convert width to integer");
                    return;
                }
            };

            let height_obj = match stream.dict.get(b"Height") {
                Ok(h) => h,
                Err(_e) => {
                    error!("Failed to get png height object");
                    return;
                }
            };

            let height = match height_obj.as_i64() {
                Ok(i) => i,
                Err(_e) => {
                    error!("Failed to convert height to integer");
                    return;
                }
            };

            let bits_obj = match stream.dict.get(b"BitsPerComponent") {
                Ok(b) => b,
                Err(_e) => {
                    error!("Failed to get bits per component");
                    return;
                }
            };

            let bits = match bits_obj.as_i64() {
                Ok(i) => i,
                Err(_e) => {
                    error!("Failed to convert bits obj to integer");
                    return;
                }
            };

            let color_obj = match stream.dict.get(b"ColorSpace") {
                Ok(s) => s,
                Err(_e) => {
                    error!("Failed to get png color object");
                    return;
                }
            };

            let color_obj_enum = match color_obj.as_name() {
                Ok(s) => PDFConColorSpace::from_pdf_format((s, bits as u8)),
                Err(_e) => {
                    error!("Failed to get color enum");
                    return;
                }
            };

            match crate::pdf_image::decompress_and_save_png(
                &stream.content,
                page_num,
                width as u32,
                height as u32,
                color_obj_enum,
                &self.out_directory,
                self.optimize,
            ) {
                Ok(_) => {}
                Err(_e) => {
                    error!("Failed to encode and save png");
                    return;
                }
            }
        }
    }

    pub fn find_xobject_images_in_page(
        &self,
        doc: &Document,
        page_num: u32,
        page_dict: &Dictionary,
    ) {
        if let Ok(resources_obj) = page_dict.get(b"Resources") {
            if let Ok(resources_dict) = resources_obj.as_dict() {
                if let Ok(x_obj) = resources_dict.get(b"XObject") {
                    if let Ok(x_obj_dict) = x_obj.as_dict() {
                        for (_name, x_ref) in x_obj_dict.iter() {
                            self.process_xobject(&doc, page_num, &x_ref);
                        }
                    } else {
                        error!("Failed to convert xobject to dict");
                    }
                } else {
                    error!("Failed to get xobject");
                }
            } else {
                error!("Failed to convert resources object to dict");
            }
        } else {
            error!("Failed to get resources object");
        }
    }

    pub fn extract_images(&self, doc: &Document) {
        doc.get_pages()
            .into_par_iter()
            .collect::<Vec<_>>()
            .par_iter()
            .progress()
            .for_each(|(page_num, page_id)| {
                if let Ok(page) = doc.get_object(*page_id) {
                    if let Ok(page_dict) = page.as_dict() {
                        self.find_xobject_images_in_page(&doc, *page_num, &page_dict);
                    }
                } else {
                    error!("Failed to get page object");
                }
            });
    }
}

impl Run for Unpack {
    fn run(&self) -> Result<(), PDFConError> {
        std::fs::create_dir_all(&self.out_directory)?;
        let document = Document::load_filtered(&self.in_file, filter_func).unwrap();
        self.extract_images(&document);

        Ok(())
    }
}
