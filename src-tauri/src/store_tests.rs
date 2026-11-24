use super::*;
use steward_core::model::Preview;

fn run(n: u64) -> RunResult {
    RunResult {
        started_at: format!("t{n}"),
        duration_ms: n,
        freed_bytes: 0,
        preview: Preview::default(),
        errors: vec![],
    }
}

#[test]
fn runs_are_newest_first_and_capped() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("nested"));
    assert!(store.load_runs().is_empty());
    for n in 0..(MAX_RUNS as u64 + 5) {
        store.push_run(run(n)).unwrap();
    }
    let runs = store.load_runs();
    assert_eq!(runs.len(), MAX_RUNS);
    assert_eq!(
        (runs[0].duration_ms, runs[MAX_RUNS - 1].duration_ms),
        (MAX_RUNS as u64 + 4, 5)
    );
}

#[test]
fn corrupt_files_are_a_cache_miss() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("overview.json"), "{ nope").unwrap();
    std::fs::write(dir.path().join("runs.json"), "[1,2").unwrap();
    let store = Store::new(dir.path().to_path_buf());
    assert!(store.load_overview().is_none());
    store.push_run(run(1)).unwrap();
    assert_eq!(store.load_runs().len(), 1);
}
