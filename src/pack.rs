use crate::pdf_image;
use crate::{Run, error::PDFConError};
use console::Style;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use log::error;
use lopdf::content::Content;
use lopdf::{Document, Object, Stream, content::Operation, dictionary};
use rayon::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pack {
    pub optimize: bool,
    pub threads: usize,
    pub in_directory: PathBuf,
    pub out_file: PathBuf,
}

#[derive(Debug)]
pub enum ImageType {
    PNG,
    JPG,
}

#[derive(Debug)]
pub struct ImageFile {
    pub location: PathBuf,
    pub image_type: ImageType,
}

impl ImageFile {
    pub fn new(location: PathBuf, image_type: ImageType) -> Self {
        Self {
            location,
            image_type,
        }
    }
}

impl Pack {
    fn optimize(
        &self,
        image_file: &ImageFile,
    ) -> Result<pdf_image::optimize::ImageData, PDFConError> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(image_file.location.to_owned())?;
        match image_file.image_type {
            ImageType::PNG => pdf_image::optimize::process_png_optimized(file),
            ImageType::JPG => pdf_image::optimize::optimize_jpeg(file),
        }
    }

    fn read_file(
        &self,
        image_file: &ImageFile,
    ) -> Result<pdf_image::optimize::ImageData, PDFConError> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(image_file.location.to_owned())?;
        match image_file.image_type {
            ImageType::PNG => pdf_image::optimize::process_png_optimized(file),
            ImageType::JPG => pdf_image::optimize::jpeg(file),
        }
    }

    fn image_file_from_entry(
        &self,
        entry: Result<std::fs::DirEntry, std::io::Error>,
    ) -> Option<ImageFile> {
        let unwrapped_entry = match entry {
            Ok(e) => e,
            //TODO Handle better
            Err(_e) => {
                // Error. Entry couldn't be read. This should be logged
                error!("Failed to create image file from entry");
                return None;
            }
        };

        let file_type = match unwrapped_entry.file_type() {
            Ok(ft) => ft,
            //TODO Handle Better
            Err(_e) => {
                // Error: This should be handled or logged
                error!("Failed to get file type");
                return None;
            }
        };

        if !file_type.is_file() {
            // Not a file. This should be logged
            return None;
        }

        let path = unwrapped_entry.path();

        let image_type = match path.extension()?.to_str()? {
            "png" => ImageType::PNG,
            "jpeg" | "jpg" => ImageType::JPG,
            _ => {
                // File was not a supported image. This should be logged
                error!("File type not supported");
                return None;
            }
        };

        Some(ImageFile::new(path, image_type))
    }

    fn para_process(&self) -> Result<(), PDFConError> {
        let term = console::Term::stdout();

        let directory = std::fs::read_dir(&self.in_directory)?;

        let mut files: Vec<ImageFile> = directory
            .filter_map(|e| {
                let entry = self.image_file_from_entry(e)?;

                Some(entry)
            })
            .collect();

        files.par_sort_by_key(|k| k.location.to_owned());

        let pre_processed = files
            .par_iter()
            .progress()
            .with_prefix("⚡Processing Images")
            .with_style(
                ProgressStyle::with_template(
                    format!("{{prefix}}: {{wide_bar}} {{pos}}/{{len}} ({{elapsed}})",).as_str(),
                )
                .unwrap_or(ProgressStyle::default_bar()),
            )
            .with_finish(indicatif::ProgressFinish::AndClear)
            .filter_map(|image_file| {
                if self.optimize {
                    match self.optimize(image_file) {
                        Ok(bytes) => Some(bytes),
                        Err(e) => {
                            // LOG and ignore
                            error!("Failed to optimize image file: {}", e.to_string());
                            return None;
                        }
                    }
                } else {
                    // Just run optimize for now until we've created the unoptimized version
                    match self.read_file(image_file) {
                        Ok(bytes) => Some(bytes),
                        //TODO Handle better
                        Err(_e) => {
                            // LOG and ignore
                            error!("Failed to read the file");
                            return None;
                        }
                    }
                }
            })
            .collect::<Vec<pdf_image::optimize::ImageData>>();

        term.write_line(
            format!(
                "{}",
                Style::new()
                    .color256(82)
                    .bold()
                    .apply_to("Processing Finished ✓")
                    .to_string(),
            )
            .as_str(),
        )?;

        // Use the latest PDF version
        let mut doc = Document::with_version("1.7");

        // Object IDs are used for cross referencing in PDF documents.
        // lopdf helps keep track of these. They're simple integers.
        // Calls to doc.new_object_id and doc.add_object produce a new object ID
        // Pages is the root node of the page tree
        let pages_id = doc.new_object_id();

        // Content is a wrapper struct around an operations struct that contains a vector of operations
        // The operations struct contains a vector of operations that match up with a particular PDF operator and
        // operands
        // The PDF spec has more details on the operators and operands
        // Note, the operators and operands are specified in a reverse order than they actually
        // appear in the PDF file itself

        // Streams are a dictionary followed by a sequence of bytes. What the bytes represent depends on the
        // context. The stream dictionary is set internally by lopdf and normally doesn't need to be manually
        // manipulated. It contains keys such as Length, Filter, DecodeParams, etc.

        let mut page_ids = Vec::new();
        let mut parent = pages_id;
        for image_data in pre_processed {
            match image_data {
                pdf_image::optimize::ImageData::PNG(compressed_data, width, height, color_type) => {
                    let (color_type, bits) = color_type.to_pdf_format();
                    let dic = dictionary!(
                        "Type" => Object::Name(b"XObject".to_vec()),
                        "Subtype" => Object::Name(b"Image".to_vec()),
                        "Width" => width as u32,
                        "Height" => height as u32,
                        "ColorSpace" => Object::Name(color_type),
                        "BitsPerComponent" => bits as u32,
                        "Filter" => Object::Name(b"FlateDecode".to_vec())
                    );
                    let img_object = Stream::new(dic, compressed_data);
                    let img_id = doc.add_object(img_object);
                    let img_name = format!("X{}", img_id.0);

                    let cm_operation = Operation::new(
                        "cm",
                        vec![
                            (width as u32).into(),
                            0.into(),
                            0.into(),
                            (height as u32).into(),
                            0.into(),
                            0.into(),
                        ],
                    );

                    let do_operation =
                        Operation::new("Do", vec![Object::Name(img_name.as_bytes().to_vec())]);
                    let content = Content {
                        operations: vec![cm_operation, do_operation],
                    };

                    let content_id =
                        doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));

                    let page_id = doc.add_object(dictionary! {
                        "Type" => "Page",
                        "Parent" => parent,
                        "Contents" => content_id,
                        "MediaBox" => vec![0.into(), 0.into(), (width as u32).into(), (height as u32).into()]
                    });

                    doc.add_xobject(page_id, img_name.as_bytes(), img_id)
                        .unwrap();

                    page_ids.push(page_id);
                    parent = page_id;
                }
                pdf_image::optimize::ImageData::JPEG(
                    compressed_data,
                    width,
                    height,
                    color_type,
                ) => {
                    let (color_type, bits) = color_type.to_pdf_format();
                    let dic = dictionary!(
                        "Type" => Object::Name(b"XObject".to_vec()),
                        "Subtype" => Object::Name(b"Image".to_vec()),
                        "Width" => width as u32,
                        "Height" => height as u32,
                        "ColorSpace" => Object::Name(color_type),
                        "BitsPerComponent" => bits as u32,
                        "Filter" => Object::Name(b"DCTDecode".to_vec())
                    );
                    let img_object = Stream::new(dic, compressed_data);
                    let img_id = doc.add_object(img_object);
                    let img_name = format!("X{}", img_id.0);

                    let cm_operation = Operation::new(
                        "cm",
                        vec![
                            (width as u32).into(),
                            0.into(),
                            0.into(),
                            (height as u32).into(),
                            0.into(),
                            0.into(),
                        ],
                    );

                    let do_operation =
                        Operation::new("Do", vec![Object::Name(img_name.as_bytes().to_vec())]);
                    let content = Content {
                        operations: vec![cm_operation, do_operation],
                    };

                    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode()?));

                    let page_id = doc.add_object(dictionary! {
                        "Type" => "Page",
                        "Parent" => parent,
                        "Contents" => content_id,
                        "MediaBox" => vec![0.into(), 0.into(), (width as u32).into(), (height as u32).into()]
                    });

                    doc.add_xobject(page_id, img_name.as_bytes(), img_id)
                        .unwrap();

                    page_ids.push(page_id);
                    parent = page_id;
                }
            }
        }

        let pages_dict = dictionary! {
            "Type" => "Pages",
            "Count" => page_ids.len() as u32,
            "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        };

        doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });

        doc.trailer.set("Root", catalog_id);

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.out_file)?;
        let mut writer = BufWriter::new(file);

        doc.save_to(&mut writer)?;

        Ok(())
    }
}

impl Run for Pack {
    fn run(&self) -> Result<(), PDFConError> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads)
            .build_global()?;
        self.para_process()?;
        Ok(())
    }
}
