use crate::utils::entry::HashTimeEntry;
use hashtime::{CompareField, Diff, DiffType, FileHashTimeResult};
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn parse_meta_file(
    meta_file_path: &PathBuf,
    format: Option<&str>,
) -> anyhow::Result<Vec<HashTimeEntry>> {
    let content = std::fs::read_to_string(meta_file_path)?;

    let fmt = if let Some(f) = format {
        f.to_string()
    } else {
        match meta_file_path.extension().and_then(|e| e.to_str()) {
            Some("csv") => "csv".to_string(),
            Some("jsonl") => "jsonl".to_string(),
            Some("json") => "json".to_string(),
            Some("txt") => "text".to_string(),
            _ => "json".to_string(),
        }
    };

    let entries: Vec<HashTimeEntry> = match fmt.as_str() {
        "csv" => {
            let mut rdr = csv::Reader::from_reader(content.as_bytes());
            let mut iter = rdr.deserialize::<HashTimeEntry>();
            match iter.next() {
                Some(Ok(e)) => vec![e],
                Some(Err(e)) => return Err(anyhow::anyhow!("Invalid CSV: {}", e)),
                None => vec![],
            }
        }
        "json" => serde_json::from_str(&content)?,
        "jsonl" => content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect(),
        "text" => content
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 7 {
                    Some(HashTimeEntry {
                        path: parts[0].to_string(),
                        size: parts[1].parse().ok(),
                        birthtime_ns: parts[2].parse().ok(),
                        mtime_ns: parts[3].parse().ok(),
                        md5: if parts[4].is_empty() {
                            None
                        } else {
                            Some(parts[4].to_string())
                        },
                        sha1: if parts[5].is_empty() {
                            None
                        } else {
                            Some(parts[5].to_string())
                        },
                        sha256: if parts[6].is_empty() {
                            None
                        } else {
                            Some(parts[6].to_string())
                        },
                        sha512: if parts.len() > 7 && !parts[7].is_empty() {
                            Some(parts[7].to_string())
                        } else {
                            None
                        },
                    })
                } else {
                    None
                }
            })
            .collect(),
        _ => anyhow::bail!("Unsupported format: {}", fmt),
    };

    Ok(entries)
}

pub fn output_jsonl_entry(entry: &HashTimeEntry, writer: &mut dyn Write) -> io::Result<()> {
    let json = serde_json::to_string(entry)?;
    writeln!(writer, "{}", json)?;
    Ok(())
}

#[allow(dead_code)]
pub fn output_csv(entries: &[HashTimeEntry], writer: &mut dyn Write) -> io::Result<()> {
    let mut wtr = csv::Writer::from_writer(writer);
    for entry in entries {
        wtr.serialize(entry)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn output_csv_entry(entry: &HashTimeEntry, writer: &mut dyn Write) -> io::Result<()> {
    let mut wtr = csv::Writer::from_writer(writer);
    wtr.serialize(entry)?;
    wtr.flush()?;
    Ok(())
}

#[allow(dead_code)]
pub fn output_text(entries: &[HashTimeEntry], writer: &mut dyn Write) -> io::Result<()> {
    for (i, entry) in entries.iter().enumerate() {
        output_text_entry(entry, writer, i + 1 < entries.len())?;
    }
    Ok(())
}

pub fn output_text_entry(
    entry: &HashTimeEntry,
    writer: &mut dyn Write,
    has_next: bool,
) -> io::Result<()> {
    writeln!(writer, "       file {}", entry.path)?;
    if let Some(size) = entry.size {
        writeln!(writer, "       size {}", size)?;
    }

    if let Some(bt) = entry.birthtime_ns {
        writeln!(writer, "  birthtime {}", bt)?;
    }
    if let Some(mt) = entry.mtime_ns {
        writeln!(writer, "      mtime {}", mt)?;
    }

    if let Some(ref md5) = entry.md5 {
        writeln!(writer, "        md5 {}", md5)?;
    }
    if let Some(ref sha1) = entry.sha1 {
        writeln!(writer, "       sha1 {}", sha1)?;
    }
    if let Some(ref sha256) = entry.sha256 {
        writeln!(writer, "     sha256 {}", sha256)?;
    }
    if let Some(ref sha512) = entry.sha512 {
        writeln!(writer, "     sha512 {}", sha512)?;
    }

    if has_next {
        writeln!(writer)?;
    }
    Ok(())
}

pub fn output_diffs(diffs: &[Diff], use_color: bool) {
    let stdout = StandardStream::stdout(if use_color {
        ColorChoice::Always
    } else {
        ColorChoice::Never
    });

    let mut stdout = stdout;

    let mut has_changes = false;

    for diff in diffs {
        has_changes = true;

        match diff.diff_type {
            DiffType::Modified => {
                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)));
                let _ = write!(stdout, "M ");
            }
            DiffType::Added => {
                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
                let _ = write!(stdout, "A ");
            }
            DiffType::Removed => {
                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
                let _ = write!(stdout, "D ");
            }
        }
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)));
        let _ = writeln!(stdout, "{}", diff.path);

        let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
        for field_diff in &diff.field_diffs {
            let field_name = match field_diff.field {
                CompareField::Md5 => "md5",
                CompareField::Sha1 => "sha1",
                CompareField::Sha256 => "sha256",
                CompareField::Sha512 => "sha512",
                CompareField::Birthtime => "birthtime",
                CompareField::Mtime => "mtime",
                CompareField::Size => "size",
            };

            if field_diff.base.is_empty() {
                let _ = writeln!(stdout, "    {}: (new) -> {}", field_name, field_diff.target);
            } else if field_diff.target.is_empty() {
                let _ = writeln!(
                    stdout,
                    "    {}: {} -> (removed)",
                    field_name, field_diff.base
                );
            } else {
                let _ = writeln!(
                    stdout,
                    "    {}: {} -> {}",
                    field_name, field_diff.base, field_diff.target
                );
            }
        }
        let _ = writeln!(stdout);
    }

    let _ = stdout.set_color(&ColorSpec::new());

    if has_changes {
        let modified = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::Modified)
            .count();
        let added = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::Added)
            .count();
        let removed = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::Removed)
            .count();

        if use_color {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
        }
        if modified > 0 {
            println!("{} modified", modified);
        }
        if added > 0 {
            println!("{} added", added);
        }
        if removed > 0 {
            println!("{} removed", removed);
        }
    } else {
        if use_color {
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
        }
        println!("No changes");
    }
}

pub fn parse_ignore_fields(ignore_fields: Option<&str>) -> HashSet<CompareField> {
    ignore_fields
        .map(|s| {
            s.split(',')
                .filter_map(|s| CompareField::from_str(s.trim()).ok())
                .collect()
        })
        .unwrap_or_default()
}

pub fn entries_to_results(
    entries: &[HashTimeEntry],
) -> (Vec<FileHashTimeResult>, Vec<String>, Vec<String>) {
    let first_entry = entries.first().unwrap();

    let hash_fields: Vec<String> = first_entry
        .md5
        .as_ref()
        .map(|_| "md5".to_string())
        .into_iter()
        .chain(first_entry.sha1.as_ref().map(|_| "sha1".to_string()))
        .chain(first_entry.sha256.as_ref().map(|_| "sha256".to_string()))
        .chain(first_entry.sha512.as_ref().map(|_| "sha512".to_string()))
        .collect();

    let time_fields: Vec<String> = first_entry
        .birthtime_ns
        .as_ref()
        .map(|_| "birthtime".to_string())
        .into_iter()
        .chain(first_entry.mtime_ns.as_ref().map(|_| "mtime".to_string()))
        .collect();

    let results: Vec<FileHashTimeResult> = entries
        .iter()
        .map(|e| FileHashTimeResult {
            path: PathBuf::from(&e.path),
            size: e.size,
            md5: e.md5.clone(),
            sha1: e.sha1.clone(),
            sha256: e.sha256.clone(),
            sha512: e.sha512.clone(),
            created_ns: e.birthtime_ns,
            modified_ns: e.mtime_ns,
        })
        .collect();

    (results, hash_fields, time_fields)
}

pub fn compute_ignored_fields(
    ignore_fields: Option<&str>,
    hashes_only: bool,
    times_only: bool,
) -> HashSet<CompareField> {
    let mut ignored = parse_ignore_fields(ignore_fields);

    if hashes_only {
        ignored.insert(CompareField::Birthtime);
        ignored.insert(CompareField::Mtime);
        ignored.insert(CompareField::Size);
    } else if times_only {
        ignored.insert(CompareField::Md5);
        ignored.insert(CompareField::Sha1);
        ignored.insert(CompareField::Sha256);
        ignored.insert(CompareField::Sha512);
    }

    ignored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ignore_fields_single() {
        let result = parse_ignore_fields(Some("md5"));
        assert!(result.contains(&CompareField::Md5));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_parse_ignore_fields_multiple() {
        let result = parse_ignore_fields(Some("md5,sha256,mtime"));
        assert!(result.contains(&CompareField::Md5));
        assert!(result.contains(&CompareField::Sha256));
        assert!(result.contains(&CompareField::Mtime));
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_ignore_fields_case_insensitive() {
        let result = parse_ignore_fields(Some("MD5,SHA256"));
        assert!(result.contains(&CompareField::Md5));
        assert!(result.contains(&CompareField::Sha256));
    }

    #[test]
    fn test_parse_ignore_fields_with_whitespace() {
        let result = parse_ignore_fields(Some("md5 , sha256"));
        assert!(result.contains(&CompareField::Md5));
        assert!(result.contains(&CompareField::Sha256));
    }

    #[test]
    fn test_parse_ignore_fields_invalid_skipped() {
        let result = parse_ignore_fields(Some("md5,invalid,sha256"));
        assert!(result.contains(&CompareField::Md5));
        assert!(result.contains(&CompareField::Sha256));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_ignore_fields_none() {
        let result = parse_ignore_fields(None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_ignore_fields_empty() {
        let result = parse_ignore_fields(Some(""));
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_ignored_fields_hashes_only() {
        let result = compute_ignored_fields(None, true, false);
        assert!(result.contains(&CompareField::Birthtime));
        assert!(result.contains(&CompareField::Mtime));
        assert!(result.contains(&CompareField::Size));
        assert!(!result.contains(&CompareField::Md5));
    }

    #[test]
    fn test_compute_ignored_fields_times_only() {
        let result = compute_ignored_fields(None, false, true);
        assert!(result.contains(&CompareField::Md5));
        assert!(result.contains(&CompareField::Sha1));
        assert!(result.contains(&CompareField::Sha256));
        assert!(result.contains(&CompareField::Sha512));
        assert!(!result.contains(&CompareField::Birthtime));
    }

    #[test]
    fn test_compute_ignored_fields_hashes_only_with_ignore() {
        let result = compute_ignored_fields(Some("md5"), true, false);
        assert!(result.contains(&CompareField::Birthtime));
        assert!(result.contains(&CompareField::Mtime));
        assert!(result.contains(&CompareField::Size));
        assert!(result.contains(&CompareField::Md5));
    }

    #[test]
    fn test_compute_ignored_fields_neither() {
        let result = compute_ignored_fields(Some("md5"), false, false);
        assert!(result.contains(&CompareField::Md5));
        assert!(!result.contains(&CompareField::Birthtime));
    }

    #[test]
    fn test_entries_to_results_single_entry() {
        let entries = vec![HashTimeEntry {
            path: "/test/file.txt".to_string(),
            size: Some(100),
            birthtime_ns: Some(1000),
            mtime_ns: Some(2000),
            md5: Some("abc123".to_string()),
            sha1: Some("def456".to_string()),
            sha256: None,
            sha512: None,
        }];

        let (results, hash_fields, time_fields) = entries_to_results(&entries);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, PathBuf::from("/test/file.txt"));
        assert_eq!(results[0].size, Some(100));
        assert_eq!(results[0].md5, Some("abc123".to_string()));
        assert_eq!(results[0].sha1, Some("def456".to_string()));
        assert_eq!(results[0].created_ns, Some(1000));
        assert_eq!(results[0].modified_ns, Some(2000));

        assert_eq!(hash_fields, vec!["md5", "sha1"]);
        assert_eq!(time_fields, vec!["birthtime", "mtime"]);
    }

    #[test]
    fn test_entries_to_results_multiple_entries() {
        let entries = vec![
            HashTimeEntry {
                path: "/test/file1.txt".to_string(),
                size: Some(100),
                birthtime_ns: Some(1000),
                mtime_ns: Some(2000),
                md5: Some("abc123".to_string()),
                sha1: None,
                sha256: None,
                sha512: None,
            },
            HashTimeEntry {
                path: "/test/file2.txt".to_string(),
                size: Some(200),
                birthtime_ns: Some(3000),
                mtime_ns: Some(4000),
                md5: Some("xyz789".to_string()),
                sha1: None,
                sha256: None,
                sha512: None,
            },
        ];

        let (results, hash_fields, time_fields) = entries_to_results(&entries);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].path, PathBuf::from("/test/file1.txt"));
        assert_eq!(results[1].path, PathBuf::from("/test/file2.txt"));
        assert_eq!(hash_fields, vec!["md5"]);
        assert_eq!(time_fields, vec!["birthtime", "mtime"]);
    }

    #[test]
    fn test_entries_to_results_partial_fields() {
        let entries = vec![HashTimeEntry {
            path: "/test/file.txt".to_string(),
            size: Some(100),
            birthtime_ns: None,
            mtime_ns: Some(2000),
            md5: None,
            sha1: Some("def456".to_string()),
            sha256: None,
            sha512: None,
        }];

        let (results, hash_fields, time_fields) = entries_to_results(&entries);

        assert_eq!(results[0].md5, None);
        assert_eq!(results[0].sha1, Some("def456".to_string()));
        assert_eq!(results[0].created_ns, None);
        assert_eq!(results[0].modified_ns, Some(2000));

        assert_eq!(hash_fields, vec!["sha1"]);
        assert_eq!(time_fields, vec!["mtime"]);
    }
}
