use crate::utils::format::{
    compute_ignored_fields, entries_to_results, output_diffs, parse_meta_file,
};
use anyhow::Result;
use hashtime::{compare, generate};

pub fn run_check(args: CheckArgs) -> Result<()> {
    let entries = parse_meta_file(
        &args.meta_file_path,
        if args.format.is_empty() {
            None
        } else {
            Some(&args.format)
        },
    )?;

    let ignored = compute_ignored_fields(
        args.ignore_fields.as_deref(),
        args.hashes_only,
        args.times_only,
    );

    let (base_results, hash_fields, time_fields) = entries_to_results(&entries);

    let target_results = generate(&args.paths, &hash_fields, &time_fields);

    let diffs = compare(&base_results[..], &target_results[..], &ignored);

    let use_color = args.color.unwrap_or_else(|| atty::is(atty::Stream::Stdout));

    output_diffs(&diffs, use_color);

    if !diffs.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

#[derive(clap::Args)]
pub struct CheckArgs {
    pub meta_file_path: std::path::PathBuf,

    pub paths: Vec<std::path::PathBuf>,

    #[arg(long, short = 'f', default_value = "")]
    pub format: String,

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
