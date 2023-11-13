#[macro_use]
extern crate lazy_static;

use clap::{command, Parser};
use globwalk::GlobWalkerBuilder;
use std::error::Error;
use std::path::{Path, PathBuf};

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

    /// Case insensitive glob matching
    #[clap(short = 'i', long)]
    case_insensitive: bool,
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
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "symlink not supported on windows",
    ))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let copy_mode = args.copy || cfg!(target_os = "windows");

    let glob_subject = args.subject.as_deref().unwrap_or("*");
    let glob_session = args.session.as_deref().unwrap_or("*");
    let glob_datatype = args.datatype.as_deref().unwrap_or("*");
    let glob_file = args.file.as_deref().unwrap_or("*");

    let walker = GlobWalkerBuilder::from_patterns(
        &args.path,
        &[
            &format!("sub-{glob_subject}/{glob_datatype}/{glob_file}"),
            &format!("sub-{glob_subject}/ses-{glob_session}/{glob_datatype}/{glob_file}"),
        ],
    )
    .case_insensitive(args.case_insensitive)
    .max_depth(5)
    .build()?;

    let mut copy_counter: usize = 0;

    let start_time = std::time::SystemTime::now();

    for entry in walker
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        copy_counter += 1;
        let path = entry.path().strip_prefix(&args.path).unwrap();
        match args.output.as_ref() {
            Some(output) => {
                let output = output.join(path);
                if !output.exists() {
                    std::fs::create_dir_all(output.parent().unwrap())?;
                    if copy_mode {
                        std::fs::copy(entry.path(), &output)?;
                    } else {
                        symlink(entry.path(), &output)?;
                    }
                } else {
                    println!("WARNING: \'{}\' already exists", output.display());
                }
            }
            None => {
                println!("{}", path.display());
            }
        }
    }

    let duration = start_time.elapsed().unwrap();

    let verb = if copy_mode { "copied" } else { "linked" };

    match args.output.as_ref() {
        Some(output) => println!(
            "{} files {} to '{}' in {:?}.",
            copy_counter,
            verb,
            output.display(),
            duration
        ),
        None => println!("{} files matched in {:?}.", copy_counter, duration),
    }

    Ok(())
}
