pub mod error;
pub mod run;

use std::{fs::File, io::BufReader};

pub use crate::{error::Error, run::RunArgs};

pub fn run(args: RunArgs) -> Result<(), Error> {
    println!(
        "looking for file: {:?}\ngenerating output at: {:?}",
        args.input_path, args.output_dir
    );

    let reader = BufReader::new(File::open(args.input_path)?);
    let registry = genvk_parse::parse_xml(reader)?;
    println!("registry has {} root elements", registry.items.len());

    Ok(())
}
