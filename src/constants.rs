use console::Style;
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
const TICK_SPEED: OnceLock<u64> = OnceLock::new();
static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();
static BOLD: OnceLock<Style> = OnceLock::new();
static C_GRAY: OnceLock<Style> = OnceLock::new();
static BC_YELLOW: OnceLock<Style> = OnceLock::new();
static BC_LGT_GREEN: OnceLock<Style> = OnceLock::new();
static BC_GREEN: OnceLock<Style> = OnceLock::new();
static BC_DRK_GREEN: OnceLock<Style> = OnceLock::new();

pub fn physical_cores() -> usize {
    *THREADS.get_or_init(|| num_cpus::get_physical())
}

pub fn tick_speed() -> u64 {
    *TICK_SPEED.get_or_init(|| 200)
}

pub fn current_dir() -> &'static PathBuf {
    CURRENT_DIR.get_or_init(|| std::env::current_dir().unwrap_or(PathBuf::from(".")))
}

pub fn bold() -> &'static Style {
    BOLD.get_or_init(|| Style::new().bold())
}
pub fn c_gray() -> &'static Style {
    C_GRAY.get_or_init(|| Style::new().color256(8))
}
pub fn bc_yellow() -> &'static Style {
    BC_YELLOW.get_or_init(|| Style::new().yellow().bold())
}
pub fn bc_lgt_green() -> &'static Style {
    BC_LGT_GREEN.get_or_init(|| Style::new().green().bold())
}
pub fn bc_green() -> &'static Style {
    BC_GREEN.get_or_init(|| Style::new().color256(107).bold())
}
pub fn bc_drk_green() -> &'static Style {
    BC_DRK_GREEN.get_or_init(|| Style::new().color256(65).bold())
}
