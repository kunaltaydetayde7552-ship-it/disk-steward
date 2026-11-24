//! `steward`: CLI over steward-core. Human tables by default, JSON with `--json`.

use serde::Serialize;
use std::{path::PathBuf, process::ExitCode};
use steward_core::{
    engine::Engine,
    measure::measure,
    model::{MeasurementState, Risk, Target},
    overview::{self, resolve},
    schedule,
    util::expand_tilde,
};

const USAGE: &str = "usage: steward <command> [options]

commands:
  overview             disk, savings, targets, schedule, docker
  targets              known targets with sizes and risk
  measure <path>       clone-aware size of a directory
  preview              what the engine would compress now (changes nothing)
  enable <id|path>     add a target to the engine patterns
  disable <id|path>    remove a target from the engine patterns
  apply --yes          compress the enabled targets for real
  schedule             LaunchAgents of interest

options:
  --json               print JSON
  --no-measure         skip size measurement (overview, targets)
  --yes                confirm `apply`
  --config <path>      engine config (default ~/.config/auto-compress-apfs/default.toml)
  --engine <path>      auto-compress-apfs binary (default ~/.cargo/bin/auto-compress-apfs)";

struct Args {
    json: bool,
    measure: bool,
    yes: bool,
    rest: Vec<String>,
    engine: Engine,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        json: false,
        measure: true,
        yes: false,
        rest: vec![],
        engine: Engine::new(),
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--json" => a.json = true,
            "--no-measure" => a.measure = false,
            "--yes" => a.yes = true,
            "--config" => {
                a.engine.config = PathBuf::from(it.next().ok_or("--config needs a path")?)
            }
            "--engine" => a.engine.bin = PathBuf::from(it.next().ok_or("--engine needs a path")?),
            "-h" | "--help" => return Err(USAGE.into()),
            _ => a.rest.push(arg),
        }
    }
    Ok(a)
}

fn main() -> ExitCode {
    match parse_args().and_then(|a| run(&a)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(a: &Args) -> Result<(), String> {
    let arg = a.rest.get(1).map(String::as_str);
    match (a.rest.first().map(String::as_str), arg) {
        (Some("overview"), _) => {
            let o = overview::overview(&a.engine, a.measure)?;
            if a.json {
                return print_json(&o);
            }
            let used = o.disk.used_bytes as f64 / o.disk.total_bytes.max(1) as f64 * 100.0;
            println!(
                "Disk {}: {} used of {} ({used:.0}%), {} free",
                o.disk.volume,
                bytes(o.disk.used_bytes),
                bytes(o.disk.total_bytes),
                bytes(o.disk.free_bytes)
            );
            let s = &o.savings;
            println!(
                "Compression: {} files, {} -> {} on disk, saved {} (ratio {:.2}), {} quarantined, last run {}",
                s.files,
                bytes(s.original_bytes),
                bytes(s.on_disk_bytes),
                bytes(s.saved_bytes),
                s.ratio,
                s.quarantined,
                s.last_run_day.as_deref().unwrap_or("never")
            );
            println!("\nTargets:");
            print_targets(&o.targets);
            println!("\nSchedule:");
            print_schedule(&o.schedule);
            let d = &o.docker;
            if o.measurement.state == MeasurementState::Never {
                println!("\nDocker: not measured (run without --no-measure)");
            } else if d.available {
                println!(
                    "\nDocker: images {}, build cache {}, volumes {}, reclaimable {}",
                    bytes(d.images_bytes),
                    bytes(d.build_cache_bytes),
                    bytes(d.volumes_bytes),
                    bytes(d.reclaimable_bytes)
                );
            } else {
                println!("\nDocker: not running");
            }
            Ok(())
        }
        (Some("targets"), _) => {
            let t = overview::targets(&a.engine, a.measure)?;
            if a.json {
                return print_json(&t);
            }
            print_targets(&t);
            Ok(())
        }
        (Some("measure"), Some(path)) => {
            let m =
                measure(&expand_tilde(path, &a.engine.home)).map_err(|e| format!("{path}: {e}"))?;
            if a.json {
                return print_json(&m);
            }
            println!(
                "files {}  compressed {}  logical {}  on disk {}  private {}",
                m.total_files,
                m.compressed_files,
                bytes(m.logical_bytes),
                bytes(m.on_disk_bytes),
                m.private_bytes.map_or("n/a".into(), bytes)
            );
            Ok(())
        }
        (Some("preview"), _) => {
            let p = a.engine.preview()?;
            if a.json {
                return print_json(&p);
            }
            println!(
                "would compress {} files, {} ({} skipped)",
                p.files,
                bytes(p.bytes),
                p.skipped
            );
            p.roots
                .iter()
                .for_each(|r| println!("  {:>10}  {}", bytes(r.bytes), r.path));
            Ok(())
        }
        (Some(cmd @ ("enable" | "disable")), Some(id)) => set_enabled(a, id, cmd == "enable"),
        (Some("apply"), _) => apply(a),
        (Some("schedule"), _) => {
            let list = schedule::list(&a.engine.home);
            if a.json {
                return print_json(&list);
            }
            print_schedule(&list);
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

fn set_enabled(a: &Args, id: &str, enable: bool) -> Result<(), String> {
    let entry = resolve(&a.engine, id)?;
    let target = overview::probe(&a.engine, &entry);
    if enable && (!target.allowed || target.risk == Risk::CloneShared) {
        return Err(format!(
            "refusing to enable {}: {}",
            target.id,
            target.risk_detail.as_deref().unwrap_or("blocked by rule")
        ));
    }
    a.engine.set_target_enabled(&entry, enable)?;
    println!(
        "{} {} ({})",
        if enable { "enabled" } else { "disabled" },
        target.id,
        target.path
    );
    Ok(())
}

fn apply(a: &Args) -> Result<(), String> {
    if !a.yes {
        return Err(
            "apply rewrites files on disk: re-run with --yes (see `steward preview` first)".into(),
        );
    }
    let blocked: Vec<Target> = overview::blocked_enabled(&a.engine)?;
    if let Some(t) = blocked.first() {
        return Err(format!(
            "refusing to apply: enabled target {} is blocked ({}); run `steward disable {}`",
            t.id,
            t.risk_detail.as_deref().unwrap_or("blocked by rule"),
            t.id
        ));
    }
    let r = a.engine.apply()?;
    if a.json {
        return print_json(&r);
    }
    println!(
        "compressed {} files ({}) in {} ms, freed {} bytes",
        r.preview.files,
        bytes(r.preview.bytes),
        r.duration_ms,
        r.freed_bytes
    );
    r.errors.iter().for_each(|e| eprintln!("{e}"));
    Ok(())
}

fn print_json<T: Serialize>(v: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(v).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn print_targets(targets: &[Target]) {
    println!(
        "{:<16} {:<3} {:<3} {:<15} {:>10} {:>10} {:>10} {:>8}  PATH",
        "ID", "ON", "OK", "RISK", "LOGICAL", "ON DISK", "PRIVATE", "FILES"
    );
    for t in targets {
        let risk = serde_json::to_value(t.risk)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        println!(
            "{:<16} {:<3} {:<3} {:<15} {:>10} {:>10} {:>10} {:>8}  {}",
            t.id,
            yn(t.enabled),
            yn(t.allowed),
            risk,
            bytes(t.logical_bytes),
            bytes(t.on_disk_bytes),
            t.private_bytes.map_or("n/a".into(), bytes),
            t.total_files,
            t.path
        );
        if let Some(d) = &t.risk_detail {
            println!("{:<16} {d}", "");
        }
    }
}

fn print_schedule(list: &[steward_core::model::ScheduleAgent]) {
    for s in list {
        let exit = s.last_exit_status.map_or("-".into(), |c| c.to_string());
        println!(
            "{:<28} loaded={:<5} last exit={:<3} log={}",
            s.label,
            s.loaded,
            exit,
            s.log_path.as_deref().unwrap_or("-")
        );
    }
}

fn yn(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

fn bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let (mut v, mut i) = (n as f64, 0);
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{n} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}
