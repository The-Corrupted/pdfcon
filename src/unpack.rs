use crate::Run;
use crate::constants::IGNORE_LIST;
use crate::error::PDFConError;
use log::error;
use lopdf::{Document, Object, ObjectId};
use oxipng::optimize_from_memory;
use rayon::prelude::*;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpack {
    pub threads: usize,
    pub out_directory: PathBuf,
    pub in_file: PathBuf,
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

impl Unpack {}

impl Run for Unpack {
    fn run(&self) -> Result<(), PDFConError> {
        let document = Document::load_filtered(&self.in_file, filter_func).unwrap();
        document
            .get_pages()
            .into_par_iter()
            .for_each(|(page_num, page_id): (u32, ObjectId)| {
                let page = match document.get_object(page_id) {
                    Ok(obj) => obj,
                    Err(e) => {
                        eprintln!(
                            "Page {}: Failed to get object {:?}: {}",
                            page_num, page_id, e
                        );
                        return;
                    }
                };

                let dict = match page.as_dict() {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Page {}: Not a dictionary: {}", page_num, e);
                        return;
                    }
                };
            });

        Ok(())
    }
}
