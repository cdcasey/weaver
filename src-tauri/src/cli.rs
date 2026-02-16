use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub base: PathBuf,
    pub local: PathBuf,
    pub remote: PathBuf,
    pub merged: PathBuf,
}

impl CliArgs {
    pub fn parse() -> Option<CliArgs> {
        let args: Vec<String> = std::env::args().collect();
        // Filter out Tauri-specific args
        let file_args: Vec<&String> = args[1..]
            .iter()
            .filter(|a| !a.starts_with('-'))
            .collect();

        if file_args.len() >= 4 {
            Some(CliArgs {
                base: PathBuf::from(file_args[0]),
                local: PathBuf::from(file_args[1]),
                remote: PathBuf::from(file_args[2]),
                merged: PathBuf::from(file_args[3]),
            })
        } else {
            None
        }
    }
}
