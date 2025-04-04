use crate::pdf_image;
use crate::{Run, error::PDFConError};
use image;
use indicatif::ProgressBar;
use lopdf::{Document, Object, ObjectId, Stream, content, dictionary};
use rayon::prelude::*;
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
            Err(e) => {
                // Error. Entry couldn't be read. This should be logged
                println!("Failed to create image file from entry");
                return None;
            }
        };

        let file_type = match unwrapped_entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                // Error: This should be handled or logged
                println!("Failed to get file type");
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
                println!("File type not supported");
                return None;
            }
        };

        Some(ImageFile::new(path, image_type))
    }

    fn para_process(&self) -> Result<(), PDFConError> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads)
            .build_global()?;

        let directory = std::fs::read_dir(&self.in_directory)?;

        println!("Converting entries to image files");
        let mut files: Vec<ImageFile> = directory
            .filter_map(|e| {
                let entry = self.image_file_from_entry(e)?;

                Some(entry)
            })
            .collect();

        files.par_sort_by_key(|k| k.location.to_owned());

        let total = files.len();
        let pb: Arc<Mutex<ProgressBar>> = Arc::new(Mutex::new(
            ProgressBar::new(total as u64).with_prefix("Processing Images: "),
        ));
        let pre_processed = files
            .par_iter()
            .filter_map(|image_file| {
                match pb.lock() {
                    Ok(p) => p.inc(1),
                    Err(e) => {
                        println!("Mutex poisoned: {}", e.to_string());
                        pb.clear_poison();
                    }
                }
                if self.optimize {
                    match self.optimize(image_file) {
                        Ok(bytes) => Some(bytes),
                        Err(e) => {
                            // LOG and ignore
                            println!("Failed to optimize image file: {}", e.to_string());
                            return None;
                        }
                    }
                } else {
                    // Just run optimize for now until we've created the unoptimized version
                    match self.read_file(image_file) {
                        Ok(bytes) => Some(bytes),
                        Err(e) => {
                            // LOG and ignore
                            println!("Failed to read the file");
                            return None;
                        }
                    }
                }
            })
            .collect::<Vec<pdf_image::optimize::ImageData>>();

        pb.clear_poison();
        pb.lock().unwrap().finish();

        drop(pb);

        let pb = ProgressBar::new(total as u64).with_prefix("Assemblying PDF: ");

        // Use the latest PDF version
        let mut doc = Document::with_version("1.7");

        // Object IDs are used for cross referencing in PDF documents.
        // lopdf helps keep track of these. They're simple integers.
        // Calls to doc.new_object_id and doc.add_object produce a new object ID
        // Pages is the root node of the page tree
        let mut pages_id = doc.new_object_id();

        // Content is a wrapper struct around an operations struct that contains a vector of operations
        // The operations struct contains a vector of operations that match up with a particular PDF operator and
        // operands
        // The PDF spec has more details on the operators and operands
        // Note, the operators and operands are specified in a reverse order than they actually
        // appear in the PDF file itself

        // Streams are a dictionary folled by a sequence of bytes. What the bytes represent depends on the
        // context. The stream dictionary is set internally by lopdf and normally doesn't need to be manually
        // manipulated. It contains keys such as Length, Filter, DecodeParams, etc.
        //let mut page_ids: Vec<_> = Vec::new();
        for image_data in pre_processed {
            match image_data {
                pdf_image::optimize::ImageData::PNG(compressed_data, width, height, color_type) => {
                }
                pdf_image::optimize::ImageData::MOZJPEG(
                    compressed_data,
                    width,
                    height,
                    color_type,
                ) => {}
                pdf_image::optimize::ImageData::JPEG(
                    compressed_data,
                    width,
                    height,
                    color_type,
                ) => {}
            }
        }

        Ok(())
    }
}

impl Run for Pack {
    fn run(&self) -> Result<(), PDFConError> {
        self.para_process()?;
        Ok(())
    }
}
