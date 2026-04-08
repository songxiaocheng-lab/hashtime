#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use std::fs::File;
    use std::io::Write;
    use std::path::absolute;
    use tempfile::tempdir;

    #[test]
    fn test_cli_gen_single_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5")
            .arg("--times=")
            .arg(file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_json_output() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5")
            .arg("--times=")
            .arg("--format=json")
            .arg(file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_csv_output() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5")
            .arg("--times=")
            .arg("--format=csv")
            .arg(file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_jsonl_output() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5")
            .arg("--times=")
            .arg("--format=jsonl")
            .arg(file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_text_output() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5")
            .arg("--times=")
            .arg("--format=text")
            .arg(file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_to_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let output_file = dir.path().join("output.jsonl");

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5")
            .arg("--times=")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&output_file)
            .arg(file_path)
            .assert()
            .success();

        assert!(output_file.exists());
    }

    #[test]
    fn test_cli_gen_multiple_hashes() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5,sha256")
            .arg("--times=")
            .arg(file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_multiple_files() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        File::create(&file1)
            .unwrap()
            .write_all(b"content1")
            .unwrap();
        File::create(&file2)
            .unwrap()
            .write_all(b"content2")
            .unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5")
            .arg("--times=")
            .arg(&file1)
            .arg(&file2)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_directory() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        File::create(&file1)
            .unwrap()
            .write_all(b"content1")
            .unwrap();
        File::create(&file2)
            .unwrap()
            .write_all(b"content2")
            .unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes=md5")
            .arg("--times=")
            .arg(dir.path())
            .assert()
            .success();
    }

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("--help").assert().success();
    }

    #[test]
    fn test_cli_version() {
        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("--version").assert().success();
    }

    #[test]
    fn test_cli_gen_hashes_only() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes-only")
            .arg(file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_times_only() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--times-only")
            .arg(file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gen_conflict_hashes_only_with_times() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--hashes-only")
            .arg("--times=md5")
            .arg(file_path)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_gen_conflict_times_only_with_hashes() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("gen")
            .arg("--times-only")
            .arg("--hashes=md5")
            .arg(file_path)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_check_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.jsonl");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file)
            .arg(&file_path)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("check")
            .arg(&meta_file)
            .arg(&file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_check_hashes_only() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.jsonl");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file)
            .arg(&file_path)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("check")
            .arg("--hashes-only")
            .arg(&meta_file)
            .arg(&file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_check_times_only() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.jsonl");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file)
            .arg(&file_path)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("check")
            .arg("--times-only")
            .arg(&meta_file)
            .arg(&file_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_compare_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file1 = dir.path().join("meta1.jsonl");
        let mut gen_cmd1 = Command::cargo_bin("hashtime").unwrap();
        gen_cmd1
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file1)
            .arg(&file_path)
            .assert()
            .success();

        let mut file2 = File::create(&file_path).unwrap();
        file2.write_all(b"modified content").unwrap();

        let meta_file2 = dir.path().join("meta2.jsonl");
        let mut gen_cmd2 = Command::cargo_bin("hashtime").unwrap();
        gen_cmd2
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file2)
            .arg(&file_path)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("compare")
            .arg(&meta_file1)
            .arg(&meta_file2)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_compare_hashes_only() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file1 = dir.path().join("meta1.jsonl");
        let mut gen_cmd1 = Command::cargo_bin("hashtime").unwrap();
        gen_cmd1
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file1)
            .arg(&file_path)
            .assert()
            .success();

        let mut file2 = File::create(&file_path).unwrap();
        file2.write_all(b"modified content").unwrap();

        let meta_file2 = dir.path().join("meta2.jsonl");
        let mut gen_cmd2 = Command::cargo_bin("hashtime").unwrap();
        gen_cmd2
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file2)
            .arg(&file_path)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("compare")
            .arg("--hashes-only")
            .arg(&meta_file1)
            .arg(&meta_file2)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_compare_times_only() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file1 = dir.path().join("meta1.jsonl");
        let mut gen_cmd1 = Command::cargo_bin("hashtime").unwrap();
        gen_cmd1
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file1)
            .arg(&file_path)
            .assert()
            .success();

        let mut file2 = File::create(&file_path).unwrap();
        file2.write_all(b"modified content").unwrap();

        let meta_file2 = dir.path().join("meta2.jsonl");
        let mut gen_cmd2 = Command::cargo_bin("hashtime").unwrap();
        gen_cmd2
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file2)
            .arg(&file_path)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("compare")
            .arg("--times-only")
            .arg(&meta_file1)
            .arg(&meta_file2)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_restore_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.jsonl");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file)
            .arg("test.txt")
            .current_dir(&dir)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("restore")
            .arg("--yes")
            .arg(&meta_file)
            .arg(".")
            .current_dir(&dir)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_restore_with_yes_flag() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.jsonl");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file)
            .arg("test.txt")
            .current_dir(&dir)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("restore")
            .arg("-y")
            .arg(&meta_file)
            .arg(".")
            .current_dir(&dir)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_restore_json_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.json");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=json")
            .arg("--output-file")
            .arg(&meta_file)
            .arg("test.txt")
            .current_dir(&dir)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("restore")
            .arg("-f")
            .arg("json")
            .arg("--yes")
            .arg(&meta_file)
            .arg(".")
            .current_dir(&dir)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_restore_csv_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.csv");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=csv")
            .arg("--output-file")
            .arg(&meta_file)
            .arg("test.txt")
            .current_dir(&dir)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("restore")
            .arg("-f")
            .arg("csv")
            .arg("--yes")
            .arg(&meta_file)
            .arg(".")
            .current_dir(&dir)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_restore_absolute_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.jsonl");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file)
            .arg(absolute(file_path).unwrap())
            .current_dir(&dir)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("restore")
            .arg("--yes")
            .arg(&meta_file)
            .arg(".")
            .current_dir(&dir)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_restore_unsafe_debugfs_flag() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let meta_file = dir.path().join("meta.jsonl");
        let mut gen_cmd = Command::cargo_bin("hashtime").unwrap();
        gen_cmd
            .arg("gen")
            .arg("--format=jsonl")
            .arg("--output-file")
            .arg(&meta_file)
            .arg("test.txt")
            .current_dir(&dir)
            .assert()
            .success();

        let mut cmd = Command::cargo_bin("hashtime").unwrap();
        cmd.arg("restore")
            .arg("--yes")
            .arg("--unsafe-debugfs")
            .arg(&meta_file)
            .arg(".")
            .current_dir(&dir)
            .assert()
            .success();
    }
}
