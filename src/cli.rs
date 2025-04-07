use clap::{ArgAction, Command, arg, command, value_parser};
use std::path::PathBuf;
use std::ffi::OsStr;

pub fn build_command() -> clap::Command {
    let c_dir = std::env::current_dir().unwrap_or(PathBuf::from("./"));
    let dir_name = c_dir.file_name().unwrap_or(OsStr::new("./"));
    let default_name = c_dir.join(dir_name).with_extension("pdf");
  
    let command: clap::Command = command!()
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
                        .required(false),
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
                )
                .arg(
                    arg!([OPTIMIZE])
                        .short('o')
                        .long("optimize")
                        .action(ArgAction::SetTrue),
                ),
        );
}
