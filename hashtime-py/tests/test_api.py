import os
import tempfile
import hashtime


class TestGenerate:
    def test_generate_single_file(self):
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(b"test content")
            f.flush()
            path = f.name
        try:
            results = hashtime.generate(
                input_paths=[path], hash_fields=["md5"], time_fields=[]
            )
            assert len(results) == 1
            assert results[0].path == path
            assert results[0].md5 is not None
        finally:
            os.unlink(path)

    def test_generate_multiple_files(self):
        with (
            tempfile.NamedTemporaryFile(delete=False) as f1,
            tempfile.NamedTemporaryFile(delete=False) as f2,
        ):
            f1.write(b"content 1")
            f2.write(b"content 2")
            f1.flush()
            f2.flush()
            path1 = f1.name
            path2 = f2.name
        try:
            results = hashtime.generate(
                input_paths=[path1, path2], hash_fields=["md5"], time_fields=[]
            )
            assert len(results) == 2
            assert results[0].md5 != results[1].md5
        finally:
            os.unlink(path1)
            os.unlink(path2)

    def test_generate_with_time_fields(self):
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(b"test")
            f.flush()
            path = f.name
        try:
            results = hashtime.generate(
                input_paths=[path], hash_fields=[], time_fields=["mtime"]
            )
            assert len(results) == 1
            assert results[0].modified_ns is not None
        finally:
            os.unlink(path)

    def test_generate_directory(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            for i in range(3):
                with open(os.path.join(tmpdir, f"file{i}.txt"), "w") as f:
                    f.write(f"content {i}")

            results = hashtime.generate(
                input_paths=[tmpdir], hash_fields=["md5"], time_fields=[]
            )
            assert len(results) >= 3

    def test_generate_empty_input(self):
        results = hashtime.generate(input_paths=[], hash_fields=["md5"], time_fields=[])
        assert results == []


class TestGenerateWithCallback:
    def test_callback_receives_results(self):
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(b"test content")
            f.flush()
            path = f.name
        try:
            received = []

            def callback(result):
                received.append(result)

            hashtime.generate_with_callback(
                input_paths=[path],
                hash_fields=["md5"],
                time_fields=[],
                callback=callback,
            )

            assert len(received) == 1
            assert received[0].path == path
            assert received[0].md5 is not None
        finally:
            os.unlink(path)


class TestCompare:
    def test_compare_identical_files(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            file_path = os.path.join(tmpdir, "test.txt")
            with open(file_path, "w") as f:
                f.write("content")

            base = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=[]
            )
            target = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=[]
            )

            diffs = hashtime.compare(
                base_results=base, target_results=target, ignored_fields=[]
            )
            assert diffs == []

    def test_compare_modified_file(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            file_path = os.path.join(tmpdir, "test.txt")

            with open(file_path, "w") as f:
                f.write("original")

            base = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=[]
            )

            with open(file_path, "w") as f:
                f.write("modified")

            target = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=[]
            )

            diffs = hashtime.compare(
                base_results=base, target_results=target, ignored_fields=[]
            )

            assert len(diffs) == 1
            assert diffs[0].diff_type == "modified"
            assert diffs[0].path == file_path

    def test_compare_ignored_fields(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            file_path = os.path.join(tmpdir, "test.txt")

            with open(file_path, "w") as f:
                f.write("content")

            base = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=["mtime"]
            )

            import time

            time.sleep(0.1)
            os.utime(file_path, None)

            target = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=["mtime"]
            )

            diffs = hashtime.compare(
                base_results=base, target_results=target, ignored_fields=["mtime"]
            )

            assert diffs == []


class TestRestoreTimes:
    def test_restore_times(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            file_path = os.path.join(tmpdir, "test.txt")
            with open(file_path, "w") as f:
                f.write("content")

            original = hashtime.generate(
                input_paths=[file_path],
                hash_fields=[],
                time_fields=["mtime", "birthtime"],
            )[0]

            import time

            time.sleep(0.2)
            os.utime(file_path, (time.time(), time.time()))

            time_result = hashtime.FileTimeResult(
                created_ns=original.created_ns, modified_ns=original.modified_ns
            )

            hashtime.restore_times([(file_path, time_result)])

            restored = hashtime.generate(
                input_paths=[file_path], hash_fields=[], time_fields=["mtime"]
            )[0]

            assert restored.modified_ns == original.modified_ns


class TestDataTypes:
    def test_file_hash_time_result_attributes(self):
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(b"test")
            f.flush()
            path = f.name
        try:
            results = hashtime.generate(
                input_paths=[path],
                hash_fields=["md5", "sha256"],
                time_fields=["mtime", "birthtime"],
            )

            r = results[0]
            assert hasattr(r, "path")
            assert hasattr(r, "size")
            assert hasattr(r, "md5")
            assert hasattr(r, "sha256")
            assert hasattr(r, "modified_ns")
            assert hasattr(r, "created_ns")

            assert r.path == path
            assert r.size is not None
            assert r.md5 is not None
            assert r.sha256 is not None
        finally:
            os.unlink(path)

    def test_file_time_result_py(self):
        time_result = hashtime.FileTimeResult(
            created_ns=1000000000, modified_ns=2000000000
        )
        assert time_result.created_ns == 1000000000
        assert time_result.modified_ns == 2000000000

    def test_diff_py_attributes(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            file_path = os.path.join(tmpdir, "test.txt")

            with open(file_path, "w") as f:
                f.write("v1")

            base = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=[]
            )

            with open(file_path, "w") as f:
                f.write("v2")

            target = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=[]
            )

            diffs = hashtime.compare(base, target, [])

            d = diffs[0]
            assert hasattr(d, "path")
            assert hasattr(d, "diff_type")
            assert hasattr(d, "field_diffs")

            assert d.path == file_path
            assert d.diff_type == "modified"

    def test_field_diff_py_attributes(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            file_path = os.path.join(tmpdir, "test.txt")

            with open(file_path, "w") as f:
                f.write("v1")

            base = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=[]
            )

            with open(file_path, "w") as f:
                f.write("v2")

            target = hashtime.generate(
                input_paths=[file_path], hash_fields=["md5"], time_fields=[]
            )

            diffs = hashtime.compare(base, target, [])
            field_diff = diffs[0].field_diffs[0]

            assert hasattr(field_diff, "field")
            assert hasattr(field_diff, "base")
            assert hasattr(field_diff, "target")
            assert field_diff.field == "md5"
