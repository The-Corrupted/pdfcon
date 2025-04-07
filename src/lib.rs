pub mod cli;
pub mod command;
pub mod constants;
pub mod error;
pub mod pack;
pub mod pdf_image;
pub mod progress;
pub mod unpack;

pub trait Run {
    fn run(&self) -> Result<(), error::PDFConError>;
}
