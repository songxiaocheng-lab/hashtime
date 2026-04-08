use crate::utils::entry::{HashTimeEntry, file_hash_time_result_to_entry};
use crate::utils::format;
use crate::utils::path_util::{parse_hash_fields, parse_time_fields};
use anyhow::Result;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

pub fn run_gen(args: GenArgs) -> Result<()> {
    let (hash_fields, time_fields) = if args.hashes_only {
        (
            vec![
                "md5".to_string(),
                "sha1".to_string(),
                "sha256".to_string(),
                "sha512".to_string(),
            ],
            vec![],
        )
    } else if args.times_only {
        (vec![], vec!["birthtime".to_string(), "mtime".to_string()])
    } else {
        (
            parse_hash_fields(&args.hashes),
            parse_time_fields(&args.times),
        )
    };
    let hash_fields_set: HashSet<_> = hash_fields.iter().collect();
    let time_fields_set: HashSet<_> = time_fields.iter().collect();

    let format = if args.format.is_empty() {
        if args.output_file.is_some() {
            "jsonl"
        } else {
            "text"
        }
    } else {
        &args.format
    };

    if format == "json" {
        run_gen_batch(
            &args,
            &hash_fields,
            &time_fields,
            &hash_fields_set,
            &time_fields_set,
        )
    } else {
        run_gen_stream(
            &args,
            &hash_fields,
            &time_fields,
            &hash_fields_set,
            &time_fields_set,
            format,
        )
    }
}

fn run_gen_batch(
    args: &GenArgs,
    hash_fields: &[String],
    time_fields: &[String],
    hash_fields_set: &HashSet<&String>,
    time_fields_set: &HashSet<&String>,
) -> Result<()> {
    let results = hashtime::generate(&args.paths, hash_fields, time_fields);

    let entries: Vec<HashTimeEntry> = results
        .iter()
        .map(|r| file_hash_time_result_to_entry(r, hash_fields_set, time_fields_set))
        .collect();

    let json = serde_json::to_string(&entries)?;

    if let Some(output_path) = &args.output_file {
        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;
    } else {
        print!("{}", json);
    }

    Ok(())
}

fn run_gen_stream(
    args: &GenArgs,
    hash_fields: &[String],
    time_fields: &[String],
    hash_fields_set: &HashSet<&String>,
    time_fields_set: &HashSet<&String>,
    format: &str,
) -> Result<()> {
    let output: Mutex<Box<dyn Write + Send>> = if let Some(output_path) = &args.output_file {
        Mutex::new(Box::new(File::create(output_path)?))
    } else {
        Mutex::new(Box::new(std::io::stdout()))
    };

    let output_clone = output;

    hashtime::generate_with_callback(&args.paths, hash_fields, time_fields, move |r| {
        let entry = file_hash_time_result_to_entry(&r, hash_fields_set, time_fields_set);

        let result = if let Ok(mut output) = output_clone.lock() {
            let mut buffered = BufWriter::new(&mut *output);
            match format {
                "jsonl" => format::output_jsonl_entry(&entry, &mut buffered),
                "csv" => format::output_csv_entry(&entry, &mut buffered),
                "text" => format::output_text_entry(&entry, &mut buffered, true),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Unsupported format",
                )),
            }
        } else {
            Err(std::io::Error::other("Failed to lock output"))
        };
        if let Err(e) = result {
            eprintln!("Error writing output: {}", e);
        }
    });

    Ok(())
}

#[derive(clap::Args)]
pub struct GenArgs {
    pub paths: Vec<PathBuf>,

    #[arg(long, short = 'o')]
    pub output_file: Option<PathBuf>,

    #[arg(
        long,
        short = 's',
        default_value = "md5,sha1,sha256,sha512",
        conflicts_with = "hashes_only"
    )]
    pub hashes: String,

    #[arg(
        long,
        short = 't',
        default_value = "birthtime,mtime",
        conflicts_with = "times_only"
    )]
    pub times: String,

    #[arg(long, short = 'S', conflicts_with = "times")]
    pub hashes_only: bool,

    #[arg(long, short = 'T', conflicts_with = "hashes")]
    pub times_only: bool,

    #[arg(long, short = 'C')]
    pub working_dir: Option<PathBuf>,

    #[arg(long, short = 'f', default_value = "")]
    pub format: String,
}
