//! Runs the real engine, but only against a temp HOME, temp config and temp data dir.

use std::fs;
use steward_core::{engine::Engine, measure::measure};

#[test]
fn apply_compresses_only_the_temp_dir() {
    let bin = steward_core::util::home_dir().join(".cargo/bin/auto-compress-apfs");
    if !bin.exists() {
        eprintln!("SKIP: auto-compress-apfs not installed");
        return;
    }
    let home = tempfile::tempdir().unwrap();
    let data = home.path().join("data");
    fs::create_dir_all(&data).unwrap();
    for i in 0..3 {
        fs::write(
            data.join(format!("f{i}.txt")),
            "compressible text\n".repeat(20_000),
        )
        .unwrap();
    }
    let config = home.path().join("config.toml");
    fs::write(
        &config,
        format!(
            "patterns = []\nmin_age_secs = 0\nmin_free_bytes = 0\nlog_dir = \"{0}/log\"\n[compress]\nrun = true\n",
            home.path().display()
        ),
    )
    .unwrap();
    let engine = Engine {
        bin,
        config,
        home: home.path().to_path_buf(),
    };
    assert!(engine.config.starts_with(home.path()) && engine.home == home.path());

    engine.set_enabled(&data, true).unwrap();
    let preview = engine.preview().unwrap();
    assert_eq!(preview.files, 3);

    let run = engine.apply().unwrap();
    assert_eq!(run.preview.files, 3);
    assert!(run.errors.is_empty(), "{:?}", run.errors);
    let m = measure(&data).unwrap();
    assert_eq!(m.compressed_files, 3);
    assert!(m.on_disk_bytes < m.logical_bytes / 2);
    assert_eq!(m.private_bytes, Some(m.on_disk_bytes)); // compressed copies are never clones
    assert_eq!(engine.preview().unwrap().files, 0);
    assert_eq!(engine.stat().unwrap().files, 3); // state.tsv lives under the temp HOME
}
