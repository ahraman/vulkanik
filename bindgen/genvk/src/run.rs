use std::path::PathBuf;

use clap::ArgMatches;

use crate::Error;

#[derive(Debug)]
pub struct RunArgs {
    pub input_path: PathBuf,
    pub output_dir: PathBuf,
}

impl RunArgs {
    pub fn from_matches(mut matches: ArgMatches) -> Result<Self, Error> {
        Ok(Self {
            input_path: get_arg(&mut matches, "input")?,
            output_dir: get_arg(&mut matches, "output")?,
        })
    }
}

fn get_arg<T>(matches: &mut ArgMatches, arg: &'static str) -> Result<T, Error>
where
    T: Clone + Send + Sync + 'static,
{
    matches
        .remove_one::<T>(arg)
        .ok_or_else(|| Error::MissingArg(arg))
}
