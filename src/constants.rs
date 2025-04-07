use num_cpus;
use std::path::PathBuf;
use std::sync::OnceLock;

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

const THREADS: OnceLock<usize> = OnceLock::new();
static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn physical_cores() -> usize {
    *THREADS.get_or_init(|| num_cpus::get_physical())
}

pub fn current_dir() -> &'static PathBuf {
    CURRENT_DIR.get_or_init(|| std::env::current_dir().unwrap_or(PathBuf::from(".")))
}
