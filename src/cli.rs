use crate::constants::physical_cores;
use crate::pack::Pack;
use crate::unpack::Unpack;
use clap::{ArgAction, Command, arg, command, value_parser};
use std::{ffi::OsStr, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PDFCon {
    UNPACK(Unpack),
    PACK(Pack),
}

pub fn get_command() -> PDFCon {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("pack")
                .about("Turn images into a pdf")
                .arg(
                    arg!([OPTIMIZE])
                        .short('o')
                        .long("optimize")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    arg!([THREADS])
                        .short('t')
                        .long("threads")
                        .value_parser(value_parser!(usize))
                        .required(false),
                )
                .arg(
                    arg!([OUT_FILE])
                        .short('f')
                        .long("file")
                        .value_parser(value_parser!(PathBuf))
                        .required(false),
                )
                .arg(
                    arg!([IN_DIRECTORY])
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("unpack")
                .about("Turn pdf into images")
                .arg(
                    arg!([THREADS])
                        .short('t')
                        .long("threads")
                        .value_parser(value_parser!(usize))
                        .required(false),
                )
                .arg(
                    arg!([OUT_DIRECTORY])
                        .short('d')
                        .long("directory")
                        .value_parser(value_parser!(PathBuf))
                        .required(false),
                )
                .arg(
                    arg!([IN_FILE])
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                ),
        )
        .get_matches();

    let total_physical = physical_cores();
    match matches.subcommand() {
        Some(("pack", sub_matches)) => PDFCon::PACK(Pack {
            optimize: sub_matches.get_flag("OPTIMIZE"),
            in_directory: sub_matches
                .get_one::<PathBuf>("IN_DIRECTORY")
                .unwrap()
                .to_owned(),
            out_file: sub_matches
                .get_one::<PathBuf>("OUT_FILE")
                .unwrap_or(&PathBuf::from("output.pdf"))
                .to_owned(),
            threads: sub_matches
                .get_one::<usize>("THREADS")
                .copied()
                .unwrap_or(total_physical / 2)
                .clamp(1usize, total_physical * 2),
        }),
        Some(("unpack", sub_matches)) => PDFCon::UNPACK(Unpack {
            threads: sub_matches
                .get_one::<usize>("THREADS")
                .copied()
                .unwrap_or(total_physical / 2)
                .clamp(1usize, total_physical * 2),
            out_directory: sub_matches
                .get_one::<PathBuf>("OUT_DIRECTORY")
                .unwrap_or(&PathBuf::from("."))
                .to_owned(),
            in_file: sub_matches
                .get_one::<PathBuf>("IN_FILE")
                .unwrap()
                .to_owned(),
        }),
        _ => unreachable!(
            "Subcommands are mandatory. It should not be possible to reach this branch"
        ),
    }
}
