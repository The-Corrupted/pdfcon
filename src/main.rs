use env_logger;
use pdfcon::Run;
use pdfcon::cli;
use pdfcon::error::PDFConError;

fn main() -> Result<(), PDFConError> {
    env_logger::init();
    let command = cli::get_command();
    match command {
        cli::PDFCon::PACK(p) => p.run(),
        cli::PDFCon::UNPACK(_) => Ok(()),
    }
}
