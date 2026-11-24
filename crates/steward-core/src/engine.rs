//! Wrapper around the `auto-compress-apfs` CLI (binary and config paths are configurable).

use crate::{
    catalog::Entry,
    disk::{self, DATA_VOLUME},
    model::{Preview, PreviewRoot, RunResult, Savings},
    util::{contract_tilde, expand_tilde, home_dir, now_rfc3339},
};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::Instant,
};
use toml_edit::{Array, DocumentMut, Value};

#[derive(Debug, Clone)]
pub struct Engine {
    pub bin: PathBuf,
    pub config: PathBuf,
    /// `$HOME` given to the engine: `~` in its config, its state, log and cache live under it.
    pub home: PathBuf,
}

impl Engine {
    pub fn new() -> Self {
        Self::for_home(home_dir())
    }

    /// GUI apps get a minimal PATH, so `~/.cargo/bin` is tried before falling back to PATH lookup.
    fn for_home(home: PathBuf) -> Self {
        let cargo_bin = home.join(".cargo/bin/auto-compress-apfs");
        Self {
            bin: if cargo_bin.exists() {
                cargo_bin
            } else {
                "auto-compress-apfs".into()
            },
            config: home.join(".config/auto-compress-apfs/default.toml"),
            home,
        }
    }

    fn run(&self, args: &[&str]) -> Result<Output, String> {
        Command::new(&self.bin)
            .args(args)
            .env("HOME", &self.home)
            .output()
            .map_err(|e| format!("cannot run {}: {e}", self.bin.display()))
    }

    fn run_with_config(&self, args: &[&str]) -> Result<Output, String> {
        let config = self.config.to_string_lossy();
        self.run(&[args, &["--config", &config]].concat())
    }

    /// What `compress` (no `--apply`) would do with the enabled patterns.
    pub fn preview(&self) -> Result<Preview, String> {
        let out = self.run_with_config(&["compress", "--no-progress"])?;
        let mut preview = parse_preview(&String::from_utf8_lossy(&out.stdout))
            .ok_or_else(|| format!("unrecognized preview output: {}", lossy_tail(&out)))?;
        for r in &mut preview.roots {
            r.path = expand_tilde(&r.path, &self.home).display().to_string();
        }
        Ok(preview)
    }

    /// Runs `compress --apply` for real. `preview` in the result is what was about to be compressed.
    pub fn apply(&self) -> Result<RunResult, String> {
        let preview = self.preview()?;
        self.apply_with(preview, |_| {}, |_| {}).map(|(run, _)| run)
    }

    /// Like `apply` for a preview taken earlier, streaming the engine's log (`-v`, stderr) line by line.
    /// `on_start` gets the child pid (so the caller can SIGINT it); `on_line` every log line.
    /// Returns the result and whether the engine exited successfully.
    pub fn apply_with(
        &self,
        preview: Preview,
        on_start: impl FnOnce(u32),
        mut on_line: impl FnMut(&str),
    ) -> Result<(RunResult, bool), String> {
        let started_at = now_rfc3339();
        let before = disk::space(DATA_VOLUME).map_err(|e| e.to_string())?.1;
        let clock = Instant::now();
        let config = self.config.to_string_lossy();
        let mut child = Command::new(&self.bin)
            .args([
                "compress",
                "--apply",
                "--no-progress",
                "-v",
                "--config",
                &config,
            ])
            .env("HOME", &self.home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("cannot run {}: {e}", self.bin.display()))?;
        on_start(child.id());
        let mut errors = Vec::new();
        let mut last_line = String::new();
        let mut log = BufReader::new(child.stderr.take().ok_or("engine stderr unavailable")?);
        let mut raw = Vec::new();
        while log.read_until(b'\n', &mut raw).map_err(|e| e.to_string())? > 0 {
            let line = String::from_utf8_lossy(&raw).trim_end().to_owned();
            raw.clear();
            on_line(&line);
            if line.contains("ERROR") || line.contains("WARN") {
                errors.push(line.clone());
            }
            last_line = line;
        }
        let status = child.wait().map_err(|e| e.to_string())?;
        let duration_ms = u64::try_from(clock.elapsed().as_millis()).unwrap_or(u64::MAX);
        let after = disk::space(DATA_VOLUME).map_err(|e| e.to_string())?.1;
        if !status.success() && errors.is_empty() {
            errors.push(format!("engine exited with {status}: {last_line}"));
        }
        let run = RunResult {
            started_at,
            duration_ms,
            freed_bytes: after as i64 - before as i64,
            preview,
            errors,
        };
        Ok((run, status.success()))
    }

    /// Installs the engine's daily LaunchAgent (`install-launchd`) for this engine's config.
    pub fn install_launchd(&self) -> Result<(), String> {
        self.run_checked(&["install-launchd"])
    }

    pub fn uninstall_launchd(&self) -> Result<(), String> {
        self.run_checked(&["uninstall-launchd"])
    }

    fn run_checked(&self, args: &[&str]) -> Result<(), String> {
        let out = self.run_with_config(args)?;
        if out.status.success() {
            return Ok(());
        }
        Err(format!(
            "{} {} failed: {}",
            self.bin.display(),
            args[0],
            lossy_tail(&out)
        ))
    }

    pub fn stat(&self) -> Result<Savings, String> {
        let out = self.run_with_config(&["stat", "--top", "0"])?;
        parse_stat(&String::from_utf8_lossy(&out.stdout))
            .ok_or_else(|| format!("unrecognized stat output: {}", lossy_tail(&out)))
    }

    /// Enabled patterns exactly as written in the config (top-level `patterns` only).
    pub fn patterns(&self) -> Result<Vec<String>, String> {
        let doc = self.read_config()?;
        Ok(doc
            .get("patterns")
            .and_then(|p| p.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Enabled patterns with `~` expanded.
    pub fn enabled_paths(&self) -> Result<Vec<PathBuf>, String> {
        Ok(self
            .patterns()?
            .iter()
            .map(|p| expand_tilde(p, &self.home))
            .collect())
    }

    /// Adds or removes `path` in the top-level `patterns`, preserving comments and other sections.
    pub fn set_enabled(&self, path: &Path, enabled: bool) -> Result<(), String> {
        self.edit_list("patterns", &[contract_tilde(path, &self.home)], enabled)
    }

    /// Enables or disables a catalog entry: its pattern plus its engine excludes. The excludes go in first
    /// and out last, so the cache is never listed for compression without them.
    pub fn set_target_enabled(&self, entry: &Entry, enabled: bool) -> Result<(), String> {
        if enabled {
            self.edit_list("exclude", &entry.excludes, true)?;
        }
        self.set_enabled(&entry.path, enabled)?;
        if !enabled {
            self.edit_list("exclude", &entry.excludes, false)?;
        }
        Ok(())
    }

    /// Compiled engine-style globs (`~` expanded; `*` never crosses `/`) for measurement filtering.
    pub fn exclude_set(&self, patterns: &[String]) -> GlobSet {
        let mut set = GlobSetBuilder::new();
        for p in patterns {
            let full = expand_tilde(p, &self.home);
            if let Ok(g) = GlobBuilder::new(&full.to_string_lossy())
                .literal_separator(true)
                .build()
            {
                set.add(g);
            }
        }
        set.build().unwrap_or_else(|_| GlobSet::empty())
    }

    /// Adds (`enabled`) or removes `items` in the top-level array `key` (idempotent), preserving comments,
    /// layout and every other section. Items compare equal after `~` expansion.
    fn edit_list(&self, key: &str, items: &[String], enabled: bool) -> Result<(), String> {
        if items.is_empty() {
            return Ok(());
        }
        let mut doc = self.read_config()?;
        let list = doc
            .entry(key)
            .or_insert_with(|| toml_edit::value(Array::new()))
            .as_array_mut()
            .ok_or(format!("`{key}` in the config is not an array"))?;
        let multiline = list.iter().next().is_some_and(|v| {
            v.decor()
                .prefix()
                .and_then(|p| p.as_str())
                .is_some_and(|s| s.contains('\n'))
        });
        let mut changed = false;
        for item in items {
            let same = |v: &Value| {
                v.as_str()
                    .is_some_and(|s| expand_tilde(s, &self.home) == expand_tilde(item, &self.home))
            };
            let present = list.iter().any(same);
            match (enabled, present) {
                (true, false) => {
                    let mut v = Value::from(item.as_str());
                    if multiline {
                        v.decor_mut().set_prefix("\n    ");
                    }
                    list.push_formatted(v);
                    changed = true;
                }
                (false, true) => {
                    list.retain(|v| !same(v));
                    changed = true;
                }
                _ => {}
            }
        }
        if !changed {
            return Ok(());
        }
        write_atomic(&self.config, &doc.to_string())
    }

    /// Raw output of `check-launchd` (stdout + stderr, trimmed); the exit code is not an error.
    pub fn check_launchd(&self) -> Result<String, String> {
        let out = self.run_with_config(&["check-launchd"])?;
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
        .trim()
        .to_owned())
    }

    fn read_config(&self) -> Result<DocumentMut, String> {
        let text = fs::read_to_string(&self.config)
            .map_err(|e| format!("{}: {e}", self.config.display()))?;
        text.parse()
            .map_err(|e| format!("{}: {e}", self.config.display()))
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    let tmp = path.with_extension("toml.steward-tmp");
    fs::write(&tmp, text)
        .and_then(|()| fs::rename(&tmp, path))
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn lossy_tail(out: &Output) -> String {
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    text.trim().lines().last().unwrap_or("").to_owned()
}

/// `"3.5 GiB"` -> bytes. Binary units; the engine prints one decimal, so the result is approximate.
pub fn parse_size(text: &str) -> Option<u64> {
    let (num, unit) = text.trim().split_once(' ')?;
    let exp = ["B", "KiB", "MiB", "GiB", "TiB"]
        .iter()
        .position(|u| *u == unit.trim())?;
    Some((num.parse::<f64>().ok()? * 1024f64.powi(exp as i32)).round() as u64)
}

/// Parses `auto-compress-apfs stat` (the `key  value` block before the table).
pub fn parse_stat(text: &str) -> Option<Savings> {
    let field = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key))
            .map(str::trim)
    };
    let last_run = field("last run day").filter(|d| d.len() == 10 && d.as_bytes()[4] == b'-');
    Some(Savings {
        files: field("files")?.parse().ok()?,
        original_bytes: parse_size(field("original")?)?,
        on_disk_bytes: parse_size(field("on disk")?)?,
        saved_bytes: parse_size(field("saved")?)?,
        ratio: field("ratio")?.parse().ok()?,
        quarantined: field("quarantined")?.parse().ok()?,
        last_run_day: last_run.map(str::to_owned),
        unattributed_bytes: 0,
    })
}

/// Parses the preview of `compress`: `would compress: N files, X (K skipped)` + `top roots by size:`.
pub fn parse_preview(text: &str) -> Option<Preview> {
    let mut lines = text.lines();
    let head = lines.find_map(|l| l.strip_prefix("would compress:"))?;
    let (files, rest) = head.split_once(" files,")?;
    let (bytes, skipped) = rest.split_once('(')?;
    let roots = lines
        .skip_while(|l| !l.starts_with("top roots by size:"))
        .skip(1)
        .map_while(|l| {
            let mut parts = l.trim_start().splitn(3, ' ');
            let size = format!("{} {}", parts.next()?, parts.next()?);
            Some(PreviewRoot {
                bytes: parse_size(&size)?,
                path: parts.next()?.trim().to_owned(),
            })
        })
        .collect();
    Some(Preview {
        files: files.trim().parse().ok()?,
        bytes: parse_size(bytes)?,
        skipped: skipped.split_whitespace().next()?.parse().ok()?,
        roots,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAT: &str = include_str!("../tests/fixtures/stat.txt");
    const PREVIEW: &str = include_str!("../tests/fixtures/preview.txt");
    const PREVIEW_TILDE: &str = include_str!("../tests/fixtures/preview_tilde.txt");
    const PREVIEW_EMPTY: &str = include_str!("../tests/fixtures/preview_empty.txt");
    const CONFIG: &str = include_str!("../tests/fixtures/config.toml");

    #[test]
    fn engine_prefers_cargo_bin_over_path() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(
            Engine::for_home(home.path().into()).bin,
            PathBuf::from("auto-compress-apfs")
        );
        let bin = home.path().join(".cargo/bin/auto-compress-apfs");
        fs::create_dir_all(bin.parent().unwrap()).unwrap();
        fs::write(&bin, "").unwrap();
        assert_eq!(Engine::for_home(home.path().into()).bin, bin);
    }

    #[test]
    fn parses_stat_fixture() {
        let s = parse_stat(STAT).unwrap();
        assert_eq!((s.files, s.quarantined), (23703, 0));
        assert_eq!(s.original_bytes, 5_261_334_938); // 4.9 GiB
        assert_eq!(s.saved_bytes, parse_size("3.5 GiB").unwrap());
        assert!(s.on_disk_bytes < s.original_bytes && (s.ratio - 3.44).abs() < 1e-9);
        assert_eq!(s.last_run_day.as_deref(), Some("2026-11-24"));
    }

    #[test]
    fn stat_without_last_run_day() {
        let s = parse_stat(&STAT.replace("2026-11-24", "never")).unwrap();
        assert_eq!(s.last_run_day, None);
        assert!(parse_stat("garbage").is_none());
    }

    #[test]
    fn parses_preview_fixtures() {
        let p = parse_preview(PREVIEW).unwrap();
        assert_eq!((p.files, p.skipped), (4, 0));
        assert_eq!(p.bytes, parse_size("878.9 KiB").unwrap());
        assert_eq!(p.roots.len(), 1);
        assert!(p.roots[0].path.ends_with("/data") && p.roots[0].bytes == p.bytes);
        let t = parse_preview(PREVIEW_TILDE).unwrap();
        assert_eq!(
            (t.files, t.roots[0].path.as_str()),
            (88, "~/.cargo/registry")
        );
        assert_eq!(parse_preview(PREVIEW_EMPTY).unwrap(), Preview::default());
        assert!(parse_preview("error: nope").is_none());
    }

    fn engine_with(config: &str) -> (tempfile::TempDir, Engine) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.toml");
        fs::write(&path, config).unwrap();
        let engine = Engine {
            bin: "/nonexistent".into(),
            config: path,
            home: "/Users/x".into(),
        };
        (dir, engine)
    }

    #[test]
    fn pattern_edit_touches_only_top_level_patterns() {
        let (_d, e) = engine_with(CONFIG);
        let before_patterns = e.patterns().unwrap();
        assert_eq!(
            before_patterns,
            ["~/.rustup", "~/.cargo/registry", "~/.npm"].map(String::from)
        );
        e.set_enabled(Path::new("/Users/x/.local/share/mise"), true)
            .unwrap();
        e.set_enabled(Path::new("/Users/x/.npm"), false).unwrap();
        e.set_enabled(Path::new("/Users/x/.npm"), false).unwrap(); // idempotent
        let after = fs::read_to_string(&e.config).unwrap();
        assert_eq!(
            e.patterns().unwrap(),
            ["~/.rustup", "~/.cargo/registry", "~/.local/share/mise"].map(String::from)
        );
        // Everything except the one `patterns = [...]` line is byte-identical.
        let strip = |s: &str| {
            s.lines()
                .filter(|l| !l.starts_with("patterns = [\"~/"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(strip(&after), strip(CONFIG));
        assert!(
            after.contains("[trash]\npatterns = []")
                && after.contains("# Glob patterns of what to compress.")
        );
    }

    fn uv_entry(home: &Path) -> Entry {
        crate::catalog::builtin(home)
            .into_iter()
            .find(|e| e.id == "uv-cache")
            .unwrap()
    }

    #[test]
    fn target_excludes_are_added_once_and_removed_with_the_rest_intact() {
        let (_d, mut e) = engine_with(CONFIG);
        e.home = "/Users/x".into();
        let uv = uv_entry(&e.home);
        let doc_exclude = |e: &Engine| -> Vec<String> {
            let doc = e.read_config().unwrap();
            doc["exclude"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect()
        };
        let original = doc_exclude(&e);
        e.set_target_enabled(&uv, true).unwrap();
        e.set_target_enabled(&uv, true).unwrap(); // idempotent
        let enabled = fs::read_to_string(&e.config).unwrap();
        assert_eq!(enabled.matches("archive-v0").count(), 2);
        assert_eq!(doc_exclude(&e).len(), original.len() + 2);
        assert!(e.patterns().unwrap().contains(&"~/.cache/uv".to_owned()));
        // Only the exclude and patterns arrays changed: [trash]/[dedup] excludes and comments are intact.
        assert!(enabled.contains(
            "[dedup]\npatterns = []\nexclude = [\".git\", \"target\", \"node_modules\"]"
        ));
        assert!(enabled.contains("# Never descended into (directories) or compressed (files)"));
        e.set_target_enabled(&uv, false).unwrap();
        e.set_target_enabled(&uv, false).unwrap();
        assert_eq!(doc_exclude(&e), original);
        assert_eq!(fs::read_to_string(&e.config).unwrap(), CONFIG);
    }

    #[test]
    fn missing_patterns_key_is_created() {
        let (_d, e) = engine_with("# only a comment\nverify = true\n");
        e.set_enabled(Path::new("/Users/x/.npm"), true).unwrap();
        let text = fs::read_to_string(&e.config).unwrap();
        assert!(text.contains("# only a comment") && text.contains("verify = true"));
        assert_eq!(e.patterns().unwrap(), ["~/.npm"]);
    }
}
