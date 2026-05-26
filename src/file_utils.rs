use std::{path::PathBuf, sync::OnceLock};

pub(crate) static DIR_PATH: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn get_dir_path(args: &[String]) -> Option<PathBuf> {
    args.windows(2)
        .find(|w| w[0] == "--directory")
        .map(|w| PathBuf::from(&w[1]))
}
