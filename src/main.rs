use env_logger;
use pdfcon::Run;
use pdfcon::command;
use pdfcon::error::PDFConError;

fn main() -> Result<(), PDFConError> {
    env_logger::init();
    let command = command::get_command();
    match command {
        command::PDFCon::PACK(p) => p.run(),
        command::PDFCon::UNPACK(up) => up.run(),
    }
}
