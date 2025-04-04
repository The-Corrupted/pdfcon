use num_cpus;
use std::cell::OnceCell;
use std::path::PathBuf;

pub static IGNORE_LIST: &[&[u8]] = &[
    b"Length",
    b"BBox",
    b"FormType",
    b"Matrix",
    b"Length1",
    b"Length2",
    b"Length3",
    b"PTEX.FileName",
    b"PREX.PageNumber",
    b"PTEX.InfoDict",
    b"FontDescriptor",
    b"ExtGState",
    b"MediaBox",
    b"Annot",
];

const THREADS: OnceCell<usize> = OnceCell::new();
const CURRENT_DIR: OnceCell<PathBuf> = OnceCell::new();

pub fn physical_cores() -> usize {
    *THREADS.get_or_init(|| num_cpus::get_physical())
}

pub fn current_dir() -> PathBuf {
    CURRENT_DIR
        .get_or_init(|| std::env::current_dir().unwrap_or(PathBuf::from(".")))
        .clone()
}
