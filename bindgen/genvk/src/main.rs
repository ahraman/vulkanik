use std::path::PathBuf;

use clap::{arg, command, value_parser};
use genvk::{Error, RunArgs};

fn main() -> Result<(), Error> {
    let matches = command!()
        .arg(
            arg!(input: -i --input <FILE> "Path to the input registry file")
                .default_value("vk.xml")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(output: -o --ouput <DIR> "Path to the root of the output directory")
                .default_value("src/generated")
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    genvk::run(RunArgs::from_matches(matches)?)
}
