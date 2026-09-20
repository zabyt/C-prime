use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cprime-compiler"))
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_cp(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_cp(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("cp") {
            out.push(path);
        }
    }
}

#[test]
fn every_example_passes_front_end_check() {
    let mut files = Vec::new();
    collect_cp(&repo().join("examples"), &mut files);
    assert!(!files.is_empty(), "no .cp examples found");
    for file in files {
        let out = Command::new(bin()).arg(&file).arg("--check").output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            out.status.success(),
            "`--check` failed for {}:\n{stderr}",
            file.display()
        );
    }
}

#[cfg(feature = "codegen")]
#[test]
fn demo_examples_compile_and_run() {
    let runnable = [
        "advanced_demo",
        "beginner_example",
        "calculator",
        "demo",
        "feature_demo",
        "hashmap_test",
        "hello_pipeline",
        "ini_test",
        "io_test",
        "json_test",
        "map_test",
        "stdlib_demo",
        "string_test",
        "sys_test",
        "vec_test",
    ];
    let runs = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("example-runs");
    std::fs::create_dir_all(&runs).unwrap();
    for name in runnable {
        let src = repo().join("examples").join(format!("{name}.cp"));
        let exe = runs.join(format!("{name}.exe"));
        let out = Command::new(bin())
            .arg(&src)
            .arg("--link")
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "link failed for {name}:\n{stderr}");

        let run = Command::new(&exe).current_dir(&runs).output().unwrap();
        let stdout = String::from_utf8_lossy(&run.stdout);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            run.status.success(),
            "run failed for {name} (exit code {:?}):\n{stdout}\n{stderr}",
            run.status.code()
        );
    }
}