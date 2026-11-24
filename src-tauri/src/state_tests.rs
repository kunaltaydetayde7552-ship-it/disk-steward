use crate::testkit::{Fixture, Recorder, wait_until};
use std::{fs, sync::Arc, time::Duration};
use steward_core::{
    model::{MeasurementState, Overview},
    util::now_rfc3339,
};

fn cached(f: &Fixture, measured_at: &str) {
    let mut o: Overview = f.state.get_overview().unwrap();
    o.measurement.measured_at = Some(measured_at.into());
    f.state.store.save_overview(&o).unwrap();
    f.state
        .store
        .save_fingerprint(&f.state.fingerprint())
        .unwrap();
}

fn touch_ahead(path: &std::path::Path) {
    let file = fs::File::options().write(true).open(path).unwrap();
    file.set_modified(std::time::SystemTime::now() + Duration::from_secs(60))
        .unwrap();
}

#[test]
fn outdated_fingerprint_marks_the_cache_stale_and_refreshes() {
    let f = Fixture::new();
    cached(&f, &now_rfc3339());
    assert_eq!(
        f.state.get_overview().unwrap().measurement.state,
        MeasurementState::Fresh
    ); // matching fingerprint
    let quiet = Arc::new(Recorder::default());
    f.state.refresh_if_outdated(quiet.clone());
    assert!(quiet.all().is_empty());

    touch_ahead(&f.state.engine.config);
    assert_eq!(
        f.state.get_overview().unwrap().measurement.state,
        MeasurementState::Stale
    );
    let out = Arc::new(Recorder::default());
    f.state.refresh_if_outdated(out.clone());
    out.wait_for("overview://updated");
    let names: Vec<_> = out.all().into_iter().map(|(e, _)| e).collect();
    assert_eq!(names, ["overview://measuring", "overview://updated"]);
    wait_until("fresh again", || {
        f.state.get_overview().unwrap().measurement.state == MeasurementState::Fresh
    });
}

#[test]
fn engine_state_mtime_and_missing_fingerprint_also_mark_stale() {
    let f = Fixture::new();
    cached(&f, &now_rfc3339());
    let state_file = f
        .state
        .engine
        .home
        .join("Library/Application Support/auto-compress-apfs/state.tsv");
    fs::create_dir_all(state_file.parent().unwrap()).unwrap();
    fs::write(&state_file, "").unwrap();
    assert_eq!(
        f.state.get_overview().unwrap().measurement.state,
        MeasurementState::Stale
    );
    cached(&f, &now_rfc3339());
    assert_eq!(
        f.state.get_overview().unwrap().measurement.state,
        MeasurementState::Fresh
    );
    fs::remove_file(f.file("appdata/overview.fingerprint")).unwrap(); // old cache files have none
    assert_eq!(
        f.state.get_overview().unwrap().measurement.state,
        MeasurementState::Stale
    );
}

#[test]
fn overview_without_cache_is_never_and_sizeless() {
    let f = Fixture::new();
    let o = f.state.get_overview().unwrap();
    assert_eq!(
        (o.measurement.state, o.measurement.measured_at),
        (MeasurementState::Never, None)
    );
    assert!(
        o.targets
            .iter()
            .all(|t| t.logical_bytes == 0 && t.private_bytes.is_none())
    );
    assert_eq!(
        o.targets
            .iter()
            .filter(|t| t.enabled)
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        ["npm"]
    );
    assert_eq!(o.savings.unattributed_bytes, 0);
    assert!(
        f.state.store.load_overview().is_none(),
        "a sizeless overview is not cached"
    );
}

#[test]
fn cached_overview_is_fresh_then_stale() {
    let f = Fixture::new();
    cached(&f, &now_rfc3339());
    assert_eq!(
        f.state.get_overview().unwrap().measurement.state,
        MeasurementState::Fresh
    );
    cached(&f, "2020-01-01T00:00:00Z");
    let o = f.state.get_overview().unwrap();
    assert_eq!(
        (o.measurement.state, o.measurement.measured_at.as_deref()),
        (MeasurementState::Stale, Some("2020-01-01T00:00:00Z"))
    );
}

#[test]
fn refresh_emits_measuring_then_updated_and_caches() {
    let f = Fixture::new();
    let out = Arc::new(Recorder::default());
    f.state.refresh_overview(out.clone());
    out.wait_for("overview://updated");
    let names: Vec<_> = out.all().into_iter().map(|(e, _)| e).collect();
    assert_eq!(names, ["overview://measuring", "overview://updated"]);
    let updated: Overview =
        serde_json::from_value(out.payloads("overview://updated").remove(0)).unwrap();
    assert_eq!(updated.measurement.state, MeasurementState::Fresh);
    // temp HOME: no target exists, so everything the engine saved is unattributed
    assert_eq!(
        updated.savings.unattributed_bytes,
        updated.savings.saved_bytes
    );
    let o = f.state.get_overview().unwrap();
    assert_eq!(o.measurement.state, MeasurementState::Fresh);
    assert!(f.file("appdata/overview.json").exists());
}

#[test]
fn refresh_failure_emits_failed_and_releases_the_guard() {
    let f = Fixture::new();
    fs::remove_file(f.file("engine.sh")).unwrap();
    let out = Arc::new(Recorder::default());
    f.state.refresh_overview(out.clone());
    out.wait_for("overview://failed");
    assert_eq!(
        out.payloads("overview://failed")[0]["code"],
        "engine_missing"
    );
    assert_eq!(f.state.get_overview().unwrap_err().code, "engine_missing");
}

#[test]
fn enabling_and_disabling_edits_only_patterns() {
    let f = Fixture::new();
    let t = f.state.set_target_enabled("mise", true).unwrap();
    assert_eq!((t.id.as_str(), t.enabled), ("mise", true));
    assert!(
        f.config_text().contains("~/.local/share/mise")
            && f.config_text().contains("[trash]\npatterns = []")
    );
    assert!(!f.state.set_target_enabled("npm", false).unwrap().enabled);
    assert!(!f.config_text().contains(".npm"));
    assert_eq!(
        f.state.set_target_enabled("nope", true).unwrap_err().code,
        "not_found"
    );
}

#[test]
fn blocked_targets_are_refused_and_the_config_is_untouched() {
    let f = Fixture::new();
    let before = f.config_text();
    for id in ["bun-store", "rust-target"] {
        assert_eq!(
            f.state.set_target_enabled(id, true).unwrap_err().code,
            "target_blocked",
            "{id}"
        );
    }
    assert_eq!(f.config_text(), before);
}

#[test]
fn clone_shared_uv_cache_is_refused() {
    let f = Fixture::new();
    let sp = f
        .state
        .engine
        .home
        .join("dev/proj/.venv/lib/python3.12/site-packages/pkg");
    fs::create_dir_all(&sp).unwrap();
    fs::write(sp.join("mod.py"), "x = 1").unwrap();
    fs::write(
        f.state.engine.home.join("dev/proj/.venv/pyvenv.cfg"),
        "home = /x\nuv = 0.9.0\n",
    )
    .unwrap();
    assert_eq!(
        f.state
            .set_target_enabled("uv-cache", true)
            .unwrap_err()
            .code,
        "clone_shared"
    );
}

#[test]
fn toggling_updates_the_cached_overview() {
    let f = Fixture::new();
    cached(&f, &now_rfc3339());
    f.state.set_target_enabled("mise", true).unwrap();
    let o = f.state.get_overview().unwrap();
    assert!(o.targets.iter().find(|t| t.id == "mise").unwrap().enabled);
}

#[test]
fn apply_streams_progress_persists_the_run_and_emits_finished() {
    let f = Fixture::new();
    let out = Recorder::default();
    let run = f.state.apply(&out).unwrap();
    assert_eq!((run.preview.files, run.errors.len()), (4, 1));
    assert!(run.errors[0].contains("WARN"));
    let phases: Vec<_> = out
        .payloads("run://progress")
        .iter()
        .map(|p| p["phase"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        (
            phases.first().map(String::as_str),
            phases.last().map(String::as_str)
        ),
        (Some("starting"), Some("done"))
    );
    let last = out.payloads("run://progress").pop().unwrap();
    assert_eq!(
        (
            last["files"].as_u64(),
            last["totalFiles"].as_u64(),
            last["bytes"].as_u64()
        ),
        (Some(2), Some(4), Some(401_000))
    );
    assert_eq!(out.payloads("run://finished").len(), 1);
    assert_eq!(f.state.store.load_runs(), [run]);
    let cfg = format!("--config {}", f.state.engine.config.display());
    assert!(
        f.calls().iter().all(|c| c.contains(&cfg)),
        "{:?}",
        f.calls()
    );
}

#[test]
fn apply_refuses_while_a_blocked_target_is_enabled() {
    let f = Fixture::with_config("patterns = [\"~/.bun/install\"]\n");
    let out = Recorder::default();
    assert_eq!(f.state.apply(&out).unwrap_err().code, "target_blocked");
    assert!(
        f.calls().iter().all(|c| !c.contains("--apply")),
        "{:?}",
        f.calls()
    );
    assert!(out.payloads("run://finished").is_empty() && f.state.store.load_runs().is_empty());
}

#[test]
fn second_apply_is_refused_and_cancel_ends_the_first_as_failed() {
    let f = Fixture::new();
    fs::write(f.file("slow"), "").unwrap();
    let out = Recorder::default();
    std::thread::scope(|s| {
        let first = s.spawn(|| f.state.apply(&out));
        wait_until("the engine to start", || f.file("started").exists());
        assert_eq!(
            f.state.apply(&Recorder::default()).unwrap_err().code,
            "run_in_progress"
        );
        f.state.cancel_run().unwrap();
        let err = first.join().unwrap().unwrap_err();
        assert_eq!(
            (err.code, err.message.as_str()),
            ("run_failed", "cancelled")
        );
    });
    let runs = f.state.store.load_runs();
    assert!(runs[0].errors.iter().any(|e| e == "cancelled"));
    assert_eq!(
        out.payloads("run://progress").pop().unwrap()["phase"],
        "failed"
    );
    assert_eq!(f.state.cancel_run().unwrap_err().code, "not_found");
    // the slot is free again
    fs::remove_file(f.file("slow")).unwrap();
    assert!(f.state.apply(&Recorder::default()).is_ok());
}

#[test]
fn schedule_commands_wrap_the_engine_with_the_temp_config() {
    let f = Fixture::new();
    assert_eq!(f.state.install_schedule().unwrap().len(), 2);
    assert_eq!(f.state.uninstall_schedule().unwrap().len(), 2);
    let cfg = f.state.engine.config.display().to_string();
    assert_eq!(
        f.calls(),
        [
            format!("install-launchd --config {cfg}"),
            format!("uninstall-launchd --config {cfg}")
        ]
    );
    fs::remove_file(f.file("engine.sh")).unwrap();
    assert_eq!(
        f.state.install_schedule().unwrap_err().code,
        "schedule_failed"
    );
}
