pub mod cli;
pub mod constants;
pub mod error;
pub mod pack;
pub mod pdf_image;
pub mod unpack;

pub trait Run {
    fn run(&self) -> Result<(), error::PDFConError>;
}
