pub mod error;
pub mod run;

pub use crate::{error::Error, run::RunArgs};

pub fn run(args: RunArgs) -> Result<(), Error> {
    println!(
        "looking for file: {:?}\ngenerating output at: {:?}",
        args.input_path, args.output_dir
    );

    Ok(())
}
