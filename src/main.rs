use pdfcon::Run;
use pdfcon::cli;
use pdfcon::error::PDFConError;

fn main() -> Result<(), PDFConError> {
    let command = cli::get_command();
    println!("command: {:?}", command);
    match command {
        cli::PDFCon::PACK(p) => p.run(),
        cli::PDFCon::UNPACK(_) => Ok(()),
    }
}
