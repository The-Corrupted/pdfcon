use crate::Run;
use crate::constants::IGNORE_LIST;
use crate::error::PDFConError;
use lopdf::Object;
use rayon::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpack {
    pub threads: usize,
    pub out_directory: PathBuf,
    pub in_file: PathBuf,
}

impl Unpack {
    fn filter_func(object_id: (u32, u16), object: &mut Object) -> Option<((u32, u16), Object)> {
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
}

impl Run for Unpack {
    fn run(&self) -> Result<(), PDFConError> {
        Ok(())
    }
}
