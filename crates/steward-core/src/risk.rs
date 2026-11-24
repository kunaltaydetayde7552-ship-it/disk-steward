//! Risk per target: none | clone_shared | rewritten_often | blocked_by_rule.

use crate::{
    catalog::{Entry, Rule},
    model::{Measure, Risk},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const MAX_DEPTH: usize = 8;
const MIB: u64 = 1 << 20;
const GIB: f64 = (1u64 << 30) as f64;
/// Shared bytes tolerated before a clone-scanned target is blocked: max(64 MiB, 10% of on-disk): compressing shared bytes
/// costs at most those bytes, while it saves ~30-50% of the private ones.
const MIN_SHARED_BYTES: u64 = 64 * MIB;
const SHARED_PERCENT: u64 = 10;
const SKIP_DIRS: [&str; 5] = ["node_modules", ".git", "target", "Library", ".Trash"];

/// `venv_roots`: where Python venvs are looked for (`~/dev`, `~/.local/share/uv/tools` in the app).
/// `sizes`: measured sizes of the entry; when they carry `private_bytes` they decide `VenvCloneScan`
/// (ground truth), otherwise the venv scan is a heuristic over uv-created venvs only.
/// Returns the risk and a one-line detail (None when there is nothing to say).
pub fn assess(
    entry: &Entry,
    venv_roots: &[PathBuf],
    sizes: Option<&Measure>,
) -> (Risk, Option<String>) {
    if let Rule::Blocked(risk, why) = &entry.rule {
        return (*risk, Some((*why).into()));
    }
    // Measured shared bytes are ground truth for every target (e.g. the pnpm store links are clones).
    if let Some((on_disk, private)) =
        sizes.and_then(|m| m.private_bytes.map(|p| (m.on_disk_bytes, p)))
    {
        return assess_measured(on_disk, private);
    }
    match &entry.rule {
        Rule::Allowed | Rule::Blocked(..) => (Risk::None, None),
        Rule::VenvCloneScan => {
            let venvs = clone_venvs(venv_roots);
            match venvs.first() {
                None => (Risk::None, None),
                Some(first) => (
                    Risk::CloneShared,
                    Some(format!(
                        "{} venv(s) hold regular files instead of symlinks (e.g. {}); compressing the cache un-shares their clones",
                        venvs.len(),
                        first.display()
                    )),
                ),
            }
        }
    }
}

fn assess_measured(on_disk: u64, private: u64) -> (Risk, Option<String>) {
    let shared = on_disk.saturating_sub(private);
    if shared <= MIN_SHARED_BYTES.max(on_disk / 100 * SHARED_PERCENT) {
        return (Risk::None, None);
    }
    (
        Risk::CloneShared,
        Some(format!(
            "{:.1} GiB are shared with APFS clones of other files; compressing would store them twice",
            shared as f64 / GIB
        )),
    )
}

/// uv-created venvs (dirs with a `uv = ` line in `pyvenv.cfg`) under `roots` whose site-packages module
/// files are regular files. Plain `python -m venv`/pip venvs share no blocks with the cache.
pub fn clone_venvs(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in roots {
        find(root, 0, &mut found);
    }
    found
}

fn find(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_DEPTH {
        return;
    }
    if dir.join("pyvenv.cfg").is_file() {
        if is_uv_venv(dir) && venv_has_regular_modules(dir) {
            out.push(dir.to_path_buf());
        }
        return; // never descend into a venv
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let skip = e
            .file_name()
            .to_str()
            .is_some_and(|n| SKIP_DIRS.contains(&n));
        if !skip && e.file_type().is_ok_and(|t| t.is_dir()) {
            find(&e.path(), depth + 1, out);
        }
    }
}

fn is_uv_venv(venv: &Path) -> bool {
    fs::read_to_string(venv.join("pyvenv.cfg"))
        .is_ok_and(|c| c.lines().any(|l| l.starts_with("uv = ")))
}

fn venv_has_regular_modules(venv: &Path) -> bool {
    let Ok(libs) = fs::read_dir(venv.join("lib")) else {
        return false;
    };
    libs.flatten()
        .any(|py| first_module_is_symlink(&py.path().join("site-packages")) == Some(false))
}

/// Looks at the first module file (`.py`/`.so`/`.pyi`) of a site-packages tree: `Some(true)` when it
/// (or a directory on the way) is a symlink, `Some(false)` when regular, `None` when there is none.
/// `.dist-info` is skipped: uv writes those as regular files even in symlink mode.
fn first_module_is_symlink(dir: &Path) -> Option<bool> {
    for e in fs::read_dir(dir).ok()?.flatten() {
        let name = e.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".dist-info") || name == "__pycache__" {
            continue;
        }
        let ft = e.file_type().ok()?;
        if ft.is_symlink() {
            return Some(true);
        }
        if ft.is_dir() {
            if let Some(r) = first_module_is_symlink(&e.path()) {
                return Some(r);
            }
        } else if matches!(
            Path::new(&*name).extension().and_then(|x| x.to_str()),
            Some("py" | "so" | "pyi")
        ) {
            return Some(false);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{catalog, model::Kind};
    use std::os::unix::fs::symlink;

    fn venv(root: &Path, name: &str, symlinked: bool) {
        venv_with_cfg(root, name, symlinked, "home = /x\nuv = 0.9.0\n");
    }

    fn venv_with_cfg(root: &Path, name: &str, symlinked: bool, cfg: &str) {
        let sp = root.join(name).join("lib/python3.12/site-packages/pkg");
        fs::create_dir_all(&sp).unwrap();
        fs::write(root.join(name).join("pyvenv.cfg"), cfg).unwrap();
        fs::create_dir_all(sp.parent().unwrap().join("pkg-1.0.dist-info")).unwrap();
        fs::write(sp.parent().unwrap().join("pkg-1.0.dist-info/METADATA"), "x").unwrap();
        if symlinked {
            let real = root.join(format!("{name}.real.py"));
            fs::write(&real, "x = 1").unwrap();
            symlink(&real, sp.join("mod.py")).unwrap();
        } else {
            fs::write(sp.join("mod.py"), "x = 1").unwrap();
        }
    }

    fn uv_entry() -> Entry {
        Entry {
            id: "uv-cache".into(),
            path: "/h/.cache/uv".into(),
            kind: Kind::PackageCache,
            rule: Rule::VenvCloneScan,
            excludes: Vec::new(),
        }
    }

    #[test]
    fn uv_is_clone_shared_only_with_regular_file_venvs() {
        let t = tempfile::tempdir().unwrap();
        let roots = [t.path().to_path_buf()];
        venv(t.path(), "linked", true);
        assert_eq!(assess(&uv_entry(), &roots, None), (Risk::None, None));
        venv(t.path(), "proj/.venv", false);
        let (risk, detail) = assess(&uv_entry(), &roots, None);
        assert_eq!(risk, Risk::CloneShared);
        assert!(detail.unwrap().contains("1 venv(s)"));
    }

    #[test]
    fn fallback_ignores_non_uv_venvs() {
        let t = tempfile::tempdir().unwrap();
        venv_with_cfg(t.path(), "pip/.venv", false, "home = /x\n");
        assert_eq!(
            assess(&uv_entry(), &[t.path().to_path_buf()], None),
            (Risk::None, None)
        );
    }

    fn measure(on_disk: u64, private: Option<u64>) -> Measure {
        Measure {
            on_disk_bytes: on_disk,
            private_bytes: private,
            ..Measure::default()
        }
    }

    #[test]
    fn measured_sizes_decide_over_the_venv_scan() {
        let t = tempfile::tempdir().unwrap();
        let roots = [t.path().to_path_buf()];
        venv(t.path(), "proj/.venv", false); // heuristic alone would say clone_shared
        let gib = 1u64 << 30;
        // 7 GiB on disk: threshold is 10% = ~716 MiB.
        let below = measure(7 * gib, Some(7 * gib - 700 * MIB));
        assert_eq!(
            assess(&uv_entry(), &roots, Some(&below)),
            (Risk::None, None)
        );
        let above = measure(7 * gib, Some(5 * gib));
        let (risk, detail) = assess(&uv_entry(), &roots, Some(&above));
        assert_eq!(risk, Risk::CloneShared);
        assert!(detail.unwrap().starts_with("2.0 GiB"));
        let above_pct = measure(7 * gib, Some(7 * gib - 720 * MIB));
        assert_eq!(
            assess(&uv_entry(), &roots, Some(&above_pct)).0,
            Risk::CloneShared
        );
        // Small cache (500 MiB): the 64 MiB floor wins over 10% = 50 MiB.
        let small = 500 * MIB;
        assert_eq!(
            assess(
                &uv_entry(),
                &roots,
                Some(&measure(small, Some(small - 60 * MIB)))
            )
            .0,
            Risk::None
        );
        assert_eq!(
            assess(
                &uv_entry(),
                &roots,
                Some(&measure(small, Some(small - 70 * MIB)))
            )
            .0,
            Risk::CloneShared
        );
        // Unknown private size falls back to the scan.
        assert_eq!(
            assess(&uv_entry(), &roots, Some(&measure(gib, None))).0,
            Risk::CloneShared
        );
    }

    #[test]
    fn measured_shared_bytes_flag_any_non_blocked_target() {
        let home = Path::new("/h");
        let pnpm = catalog::builtin(home)
            .into_iter()
            .find(|e| e.id == "pnpm-store")
            .unwrap();
        let gib = 1u64 << 30;
        assert_eq!(
            assess(&pnpm, &[], Some(&measure(3 * gib, Some(gib / 2)))).0,
            Risk::CloneShared
        );
        assert_eq!(
            assess(&pnpm, &[], Some(&measure(3 * gib, Some(3 * gib)))).0,
            Risk::None
        );
        let bun = catalog::builtin(home)
            .into_iter()
            .find(|e| e.id == "bun-store")
            .unwrap();
        assert_eq!(
            assess(&bun, &[], Some(&measure(3 * gib, Some(3 * gib)))).0,
            Risk::BlockedByRule
        );
    }

    #[test]
    fn static_rules() {
        let home = Path::new("/h");
        let by_id = |id: &str| {
            catalog::builtin(home)
                .into_iter()
                .find(|e| e.id == id)
                .unwrap()
        };
        assert_eq!(assess(&by_id("npm"), &[], None), (Risk::None, None));
        assert_eq!(
            assess(&by_id("bun-store"), &[], None).0,
            Risk::BlockedByRule
        );
        assert_eq!(
            assess(&by_id("rust-target"), &[], None).0,
            Risk::RewrittenOften
        );
    }

    #[test]
    fn scan_skips_node_modules_and_ignores_missing_roots() {
        let t = tempfile::tempdir().unwrap();
        venv(&t.path().join("node_modules"), "v", false);
        assert!(clone_venvs(&[t.path().into(), "/nonexistent-xyz".into()]).is_empty());
    }
}
