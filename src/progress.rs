use std::sync::OnceLock;

use console::Style;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use log::error;

static BOLD: OnceLock<Style> = OnceLock::new();
static C_GRAY: OnceLock<Style> = OnceLock::new();
static BC_YELLOW: OnceLock<Style> = OnceLock::new();
static BC_LGT_GREEN: OnceLock<Style> = OnceLock::new();
static BC_GREEN: OnceLock<Style> = OnceLock::new();
static BC_DRK_GREEN: OnceLock<Style> = OnceLock::new();

fn bold() -> &'static Style {
    BOLD.get_or_init(|| Style::new().bold())
}
fn c_gray() -> &'static Style {
    C_GRAY.get_or_init(|| Style::new().color256(8))
}
fn bc_yellow() -> &'static Style {
    BC_YELLOW.get_or_init(|| Style::new().yellow().bold())
}
fn bc_lgt_green() -> &'static Style {
    BC_LGT_GREEN.get_or_init(|| Style::new().green().bold())
}
fn bc_green() -> &'static Style {
    BC_GREEN.get_or_init(|| Style::new().color256(107).bold())
}
fn bc_drk_green() -> &'static Style {
    BC_DRK_GREEN.get_or_init(|| Style::new().color256(65).bold())
}

pub fn bar(prefix: &str, total_progress: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_progress);
    pb.set_style(ProgressStyle::default_bar()
                .progress_chars("█▓█")
                .tick_strings(&["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙●"])
                .template(format!(
                " {{spinner:.yellow.bold}} {{prefix:.yellow.bold}}{} {}{{wide_bar:.2.bold/:.65.bold}}{{msg}} {{percent:.green.bold}}{} {}{{pos:.8}}{}{{len:.8}}{} ",
                bold().apply_to(":").to_string(),
                bc_lgt_green().apply_to("").to_string(),
                bc_lgt_green().apply_to("%").to_string(),
                c_gray().apply_to("(").to_string(),
                c_gray().apply_to("/").to_string(),
                c_gray().apply_to(")").to_string()
            ).as_str())
        .unwrap()
    );
    pb.set_prefix(prefix.to_string());
    pb.set_message(bc_drk_green().apply_to("").to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(200));

    return pb;
}

pub fn spinner(prefix: &str) -> ProgressBar {
    let spnr = ProgressBar::new_spinner();

    spnr.set_style(ProgressStyle::default_spinner()
                .tick_strings(&["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙●"])
                .template(format!(
                " {{spinner:.yellow.bold}} {{prefix:.yellow.bold}} {{wide_msg}} {}{{elapsed:.8}}{} ",
                c_gray().apply_to("(").to_string(),
                c_gray().apply_to(")").to_string()
            ).as_str())
        .unwrap(),
    );
    spnr.set_prefix(prefix.to_string());
    spnr.enable_steady_tick(std::time::Duration::from_millis(200));

    return spnr;
}

pub fn update_bar(bar: &ProgressBar, pos: u64, total: u64) {
    if pos >= total - 2 && pos < total {
        bar.set_message(bc_green().apply_to("").to_string());
    } else if pos == total {
        bar.set_message(bc_lgt_green().apply_to("").to_string());
    }
}

pub fn close_bar(bar: ProgressBar, msg: &str) {
    bar.finish_and_clear();
    match Term::stdout().write_line(format!("{}", bc_yellow().apply_to(msg)).as_str()) {
        Ok(out) => out,
        Err(_e) => {
            error!("Failed to print to console");
        }
    }
}
