use env_logger;
use pdfcon::Run;
use pdfcon::command;
use pdfcon::error::PDFConError;
use std::ffi::OsStr;

fn main() -> Result<(), PDFConError> {
    env_logger::init();
    let command = command::get_command();
    match command {
        command::PDFCon::PACK(mut p) => {
            if p.out_file.is_dir() {
                // your dumb
                log::error!("File name is a directory!");
                std::process::exit(1);
            }
            let pdf_ext = OsStr::new("pdf");
            if p.out_file.extension().unwrap_or(OsStr::new("")) != pdf_ext {
                let temp = p.out_file.with_extension("pdf");
                p.out_file = temp;
            }
            p.run()
        }
        command::PDFCon::UNPACK(up) => up.run(),
    }
}
