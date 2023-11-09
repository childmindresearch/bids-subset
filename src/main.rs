#[macro_use]
extern crate lazy_static;

use std::{error::Error, path::PathBuf};

use clap::{command, Parser};
use globset::{Glob, GlobSetBuilder};
use std::path::Path;
use walkdir::WalkDir;

/// BIDS datasets subsetter
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to input BIDS dataset
    #[clap(index = 1)]
    path: PathBuf,

    /// Path to output BIDS dataset
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Subject glob pattern
    #[clap(short, long)]
    subject: Option<String>,

    /// Session glob pattern
    #[clap(short = 'e', long)]
    session: Option<String>,

    /// BIDS Datatype glob pattern (anat, func, ...)
    #[clap(short, long)]
    datatype: Option<String>,

    /// File filter pattern
    #[clap(short, long)]
    file: Option<String>,

    /// Exclude top level metadata files
    #[clap(short = 'x', long)]
    exclude_top_level: bool,

    /// Enable copy mode (default on linux is symlink)
    #[clap(short, long)]
    copy: bool,
}

lazy_static! {
    static ref TOP_LEVEL_FILES: Vec<&'static str> = vec![
        "CHANGES",
        "dataset_description.json",
        "participants.tsv",
        "participants.json",
        "README",
    ];
}

#[cfg(target_os = "macos")]
fn symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(target_os = "linux")]
fn symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(target_os = "windows")]
fn symlink(_: &Path, _: &Path) -> std::io::Result<()> {
    //raise error
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "symlink not supported on windows",
    ))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let glob_subject = args.subject.as_deref().unwrap_or("*");
    let glob_session = args.session.as_deref().unwrap_or("*");
    let glob_datatype = args.datatype.as_deref().unwrap_or("*");
    let glob_file = args.file.as_deref().unwrap_or("*");

    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new(&format!(
        "sub-{glob_subject}/{glob_datatype}/{glob_file}"
    ))?);
    builder.add(Glob::new(&format!(
        "sub-{glob_subject}/ses-{glob_session}/{glob_datatype}/{glob_file}"
    ))?);
    if !args.exclude_top_level {
        for f in TOP_LEVEL_FILES.iter() {
            builder.add(Glob::new(f)?);
        }
    }
    let set = builder.build()?;

    let walker = WalkDir::new(&args.path).max_depth(5).into_iter();

    let mut file_counter: usize = 0;

    for entry in walker {
        let entry = entry?;
        let path = entry
            .path()
            .strip_prefix((&args.path).clone().to_str().unwrap())
            .unwrap();
        if set.is_match(path) {
            match args.output.as_ref() {
                Some(output) => {
                    let output = output.join(path);
                    if !output.exists() {
                        std::fs::create_dir_all(output.parent().unwrap())?;
                        if args.copy || cfg!(target_os = "windows") {
                            std::fs::copy(entry.path(), &output)?;
                        } else {
                            symlink(entry.path(), &output)?;
                        }
                        file_counter += 1;
                    } else {
                        println!("WARNING: \'{}\' already exists", output.display());
                    }
                }
                None => {
                    println!("{}", path.display());
                    file_counter += 1;
                }
            }
        }
    }

    match args.output.as_ref() {
        Some(output) => println!("{} files copied to {}", file_counter, output.display()),
        None => println!("{} files found", file_counter),
    }

    Ok(())
}
