//! Assembles targets (catalog + engine patterns + risk + measurements) and the full overview.

use crate::{
    catalog::{self, Entry},
    disk, docker,
    engine::Engine,
    measure::measure_excluding,
    model::{DockerUsage, Measure, Measurement, MeasurementState, Overview, Risk, Target},
    risk, schedule,
    util::expand_tilde,
};
use std::{path::PathBuf, thread};

/// Where Python venvs are searched for the `clone_shared` rule.
pub fn venv_roots(engine: &Engine) -> Vec<PathBuf> {
    vec![
        engine.home.join("dev"),
        engine.home.join(".local/share/uv/tools"),
    ]
}

/// Catalog entries plus custom engine patterns, in display order.
pub fn entries(engine: &Engine) -> Result<Vec<Entry>, String> {
    Ok(catalog::with_custom(&engine.home, &engine.enabled_paths()?))
}

/// Resolves `<id|path>` (path may start with `~`); an unknown path becomes a custom entry.
pub fn resolve(engine: &Engine, arg: &str) -> Result<Entry, String> {
    let all = entries(engine)?;
    let path = expand_tilde(arg, &engine.home);
    if let Some(found) = all.into_iter().find(|e| e.id == arg || e.path == path) {
        return Ok(found);
    }
    if path.is_absolute() {
        return Ok(catalog::custom(path, &engine.home));
    }
    Err(format!(
        "unknown target `{arg}` (use an id from `steward targets` or an absolute path)"
    ))
}

/// Builds the target for one entry. `sizes: None` leaves every size at zero.
pub fn target_for(engine: &Engine, entry: &Entry, enabled: bool, sizes: Option<Measure>) -> Target {
    let (risk, risk_detail) = risk::assess(entry, &venv_roots(engine), sizes.as_ref());
    let m = sizes.unwrap_or_default();
    Target {
        id: entry.id.clone(),
        path: entry.path.display().to_string(),
        kind: entry.kind,
        enabled,
        allowed: !matches!(entry.rule, catalog::Rule::Blocked(..)) && risk != Risk::CloneShared,
        logical_bytes: m.logical_bytes,
        on_disk_bytes: m.on_disk_bytes,
        private_bytes: m.private_bytes,
        total_files: m.total_files,
        compressed_files: m.compressed_files,
        risk,
        risk_detail,
    }
}

/// Measures an entry without the files its engine excludes will skip, so sizes and risk match what is compressed.
fn measure_entry(engine: &Engine, entry: &Entry) -> Option<Measure> {
    measure_excluding(&entry.path, &engine.exclude_set(&entry.excludes)).ok()
}

/// Like `target_for` with `sizes: None`, but measures the target first when its risk depends on sizes
/// (every non-blocked target; the uv cache takes 10-40 s; more accurate than the venv-scan heuristic).
pub fn probe(engine: &Engine, entry: &Entry) -> Target {
    let sizes = (!matches!(entry.rule, catalog::Rule::Blocked(..)))
        .then(|| measure_entry(engine, entry))
        .flatten();
    target_for(engine, entry, false, sizes)
}

/// Enabled targets that must not be compressed. Measured like `probe` (same rule as `enable`); the
/// venv-scan heuristic only applies when a measurement fails.
pub fn blocked_enabled(engine: &Engine) -> Result<Vec<Target>, String> {
    let enabled = engine.enabled_paths()?;
    let entries: Vec<Entry> = entries(engine)?
        .into_iter()
        .filter(|e| enabled.contains(&e.path))
        .collect();
    Ok(thread::scope(|s| {
        let handles: Vec<_> = entries
            .iter()
            .map(|e| s.spawn(|| probe(engine, e)))
            .collect();
        handles
            .into_iter()
            .filter_map(|h| h.join().ok())
            .filter(|t| !t.allowed)
            .map(|t| Target { enabled: true, ..t })
            .collect()
    }))
}

/// All targets; with `with_sizes` every existing target is measured on its own thread.
pub fn targets(engine: &Engine, with_sizes: bool) -> Result<Vec<Target>, String> {
    let enabled = engine.enabled_paths()?;
    let entries = entries(engine)?;
    Ok(thread::scope(|s| {
        let handles: Vec<_> = entries
            .iter()
            .map(|e| {
                let measurable = with_sizes && !e.path.to_string_lossy().contains('*');
                let sizes = measurable.then(|| s.spawn(|| measure_entry(engine, e)));
                (e, sizes)
            })
            .collect();
        handles
            .into_iter()
            .map(|(e, sizes)| {
                let sizes = sizes.and_then(|h| h.join().ok().flatten());
                target_for(engine, e, enabled.contains(&e.path), sizes)
            })
            .collect()
    }))
}

pub fn overview(engine: &Engine, with_sizes: bool) -> Result<Overview, String> {
    let disk = disk::status().map_err(|e| e.to_string())?;
    let mut savings = engine.stat()?;
    let targets = targets(engine, with_sizes)?;
    let measurement = if with_sizes {
        savings.unattributed_bytes = unattributed_bytes(savings.saved_bytes, &targets);
        Measurement {
            state: MeasurementState::Fresh,
            measured_at: Some(disk.measured_at.clone()),
        }
    } else {
        Measurement {
            state: MeasurementState::Never,
            measured_at: None,
        }
    };
    // `docker system df` alone takes seconds, so the instant overview leaves it out.
    let docker = if with_sizes {
        docker::usage()
    } else {
        DockerUsage::default()
    };
    Ok(Overview {
        disk,
        savings,
        targets,
        schedule: schedule::list(&engine.home),
        docker,
        measurement,
    })
}

/// Engine saved bytes not explained by enabled targets, never negative.
///
/// `logical - onDisk` of a target is signed: uncompressed small files sit in whole 4 KiB blocks, so they
/// subtract from the total, while the engine only counts compressed files. The engine also keeps records of
/// files that were deleted or rewritten since (it never revisits them), which no target can account for.
pub fn unattributed_bytes(engine_saved: u64, targets: &[Target]) -> u64 {
    let attributed: i128 = targets
        .iter()
        .filter(|t| t.enabled)
        .map(|t| i128::from(t.logical_bytes) - i128::from(t.on_disk_bytes))
        .sum();
    u64::try_from((i128::from(engine_saved) - attributed).max(0)).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Kind, Risk};
    use std::fs;

    fn target(enabled: bool, logical: u64, on_disk: u64) -> Target {
        Target {
            id: "t".into(),
            path: "/t".into(),
            kind: Kind::Custom,
            enabled,
            allowed: true,
            logical_bytes: logical,
            on_disk_bytes: on_disk,
            private_bytes: None,
            total_files: 0,
            compressed_files: 0,
            risk: Risk::None,
            risk_detail: None,
        }
    }

    #[test]
    fn blocked_enabled_uses_measured_sizes_not_the_venv_heuristic() {
        let home = tempfile::tempdir().unwrap();
        let h = home.path();
        let cache = h.join(".cache/uv");
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("wheel"), vec![7u8; 100_000]).unwrap();
        let sp = h.join("dev/old/.venv/lib/python3.12/site-packages");
        fs::create_dir_all(&sp).unwrap();
        fs::write(sp.join("mod.py"), "x = 1").unwrap();
        fs::write(h.join("dev/old/.venv/pyvenv.cfg"), "uv = 0.9.0\n").unwrap();
        let config = h.join("c.toml");
        fs::write(&config, "patterns = [\"~/.cache/uv\", \"~/proj/target\"]\n").unwrap();
        let engine = Engine {
            bin: "/nonexistent".into(),
            config,
            home: h.into(),
        };
        let blocked = blocked_enabled(&engine).unwrap();
        // The regular-file venv would block uv by heuristic; the measurement says no shared bytes.
        assert_eq!(
            blocked.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
            ["custom:~/proj/target"]
        );
    }

    #[test]
    fn unattributed_uses_signed_enabled_targets_and_never_goes_negative() {
        // saved 35, attributed (100-70) + (50-55) = 25, disabled target ignored.
        let t = [
            target(true, 100, 70),
            target(true, 50, 55),
            target(false, 999, 1),
        ];
        assert_eq!(unattributed_bytes(35, &t), 10);
        assert_eq!(unattributed_bytes(20, &t), 0);
        assert_eq!(unattributed_bytes(5, &[]), 5);
    }
}
