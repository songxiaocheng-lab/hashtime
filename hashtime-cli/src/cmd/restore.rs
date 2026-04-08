use crate::utils::entry::HashTimeEntry;
use crate::utils::format::parse_meta_file;
use anyhow::Result;
use hashtime::FileTimeResult;
use std::io::Write;
use std::path::PathBuf;

pub fn run_restore(args: RestoreArgs) -> Result<()> {
    let format = if args.format.is_empty() {
        None
    } else {
        Some(args.format.as_str())
    };

    let entries: Vec<HashTimeEntry> = parse_meta_file(&args.meta_file_path, format)?;

    #[cfg(target_os = "linux")]
    {
        let has_birthtime_data = entries.iter().any(|e| e.birthtime_ns.is_some());

        if has_birthtime_data && !args.unsafe_debugfs {
            let mut entries_to_modify = entries;
            for entry in &mut entries_to_modify {
                entry.birthtime_ns = None;
            }
            eprintln!(
                "Warning: birthtime restore on Linux requires --unsafe-debugfs flag. Skipping birthtime."
            );
            eprintln!("Use --unsafe-debugfs to attempt anyway (requires root + debugfs binary).");
            return run_restore_impl(entries_to_modify, args);
        } else if has_birthtime_data && args.unsafe_debugfs {
            eprintln!(
                "Warning: --unsafe-debugfs is a low-level hack using debugfs. It requires root privileges"
            );
            eprintln!("and debugfs binary. Use at your own risk.");
        }
    }

    run_restore_impl(entries, args)
}

fn run_restore_impl(entries: Vec<HashTimeEntry>, args: RestoreArgs) -> Result<()> {
    for entry in &entries {
        if std::path::Path::new(&entry.path).is_absolute() {
            return Err(anyhow::anyhow!(
                "All paths in meta file must be relative, but found absolute path: {}",
                entry.path
            ));
        }
    }

    if !args.yes {
        print!(
            "About to restore timestamps for {} files under '{}'. Continue? [y/N] ",
            entries.len(),
            args.base_path.display()
        );
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let base_path = &args.base_path;

    let time_results: Vec<(PathBuf, FileTimeResult)> = entries
        .into_iter()
        .map(|entry| {
            let full_path = base_path.join(&entry.path);
            (
                full_path,
                FileTimeResult {
                    created_ns: entry.birthtime_ns,
                    modified_ns: entry.mtime_ns,
                },
            )
        })
        .collect();

    hashtime::restore_times(time_results);

    println!("Done restoring timestamps.");
    Ok(())
}

#[derive(clap::Args)]
pub struct RestoreArgs {
    pub meta_file_path: PathBuf,

    pub base_path: PathBuf,

    #[arg(long, short = 'f', default_value = "")]
    pub format: String,

    #[arg(long, short = 'y')]
    pub yes: bool,

    #[arg(long, short = 'C')]
    pub working_dir: Option<PathBuf>,

    /// Use debugfs to restore birthtime on Linux (requires root + debugfs binary).
    /// This is a low-level hack - use at your own risk.
    #[arg(long)]
    pub unsafe_debugfs: bool,
}
