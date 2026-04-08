use crate::utils::format::{
    compute_ignored_fields, entries_to_results, output_diffs, parse_meta_file,
};
use anyhow::Result;
use hashtime::compare;

pub fn run_compare(args: CompareArgs) -> Result<()> {
    let base_entries = parse_meta_file(&args.base_meta_file_path, None)?;
    let target_entries = parse_meta_file(&args.target_meta_file_path, None)?;

    let ignored = compute_ignored_fields(
        args.ignore_fields.as_deref(),
        args.hashes_only,
        args.times_only,
    );

    let (base_results, _, _) = entries_to_results(&base_entries);
    let (target_results, _, _) = entries_to_results(&target_entries);

    let diffs = compare(&base_results[..], &target_results[..], &ignored);

    let use_color = args.color.unwrap_or_else(|| atty::is(atty::Stream::Stdout));

    output_diffs(&diffs, use_color);

    if !diffs.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

#[derive(clap::Args)]
pub struct CompareArgs {
    pub base_meta_file_path: std::path::PathBuf,

    pub target_meta_file_path: std::path::PathBuf,

    #[arg(long, short = 'i')]
    pub ignore_fields: Option<String>,

    #[arg(long, short = 'C')]
    pub working_dir: Option<std::path::PathBuf>,

    #[arg(long, short = 'c')]
    pub color: Option<bool>,

    #[arg(long, short = 'S')]
    pub hashes_only: bool,

    #[arg(long, short = 'T')]
    pub times_only: bool,
}
