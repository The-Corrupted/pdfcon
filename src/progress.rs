use crate::constants::{bc_drk_green, bc_green, bc_lgt_green, bc_yellow, bold, c_gray};
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use log::error;

pub fn bar(prefix: &str, total_progress: u64, tick_speed: u64) -> ProgressBar {
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
        .unwrap_or(ProgressStyle::default_bar()));
    pb.set_prefix(prefix.to_string());
    pb.set_message(bc_drk_green().apply_to("").to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(tick_speed));

    return pb;
}

pub fn spinner(prefix: &str, tick_speed: u64) -> ProgressBar {
    let spnr = ProgressBar::new_spinner();

    spnr.set_style(ProgressStyle::default_spinner()
                .tick_strings(&["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙●"])
                .template(format!(
                " {{spinner:.yellow.bold}} {{prefix:.yellow.bold}} {{wide_msg}} {}{{elapsed:.8}}{} ",
                c_gray().apply_to("(").to_string(),
                c_gray().apply_to(")").to_string()
            ).as_str())
        .unwrap_or(ProgressStyle::default_spinner()));
    spnr.set_prefix(prefix.to_string());
    spnr.enable_steady_tick(std::time::Duration::from_millis(tick_speed));

    return spnr;
}

pub fn update_end_cap(bar: &ProgressBar, pos: u64, total: u64) {
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
