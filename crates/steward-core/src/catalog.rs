//! Built-in list of known dev targets plus engine patterns the catalog does not know (kind `custom`).

use crate::{
    model::{Kind, Risk},
    util::contract_tilde,
};
use std::path::{Path, PathBuf};

/// How a target may be treated, before any filesystem inspection.
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    Allowed,
    /// Allowed only while no Python venv holds clones of the cache (see `risk`).
    VenvCloneScan,
    /// Never compress: `Risk` and the one-line reason are reported as-is.
    Blocked(Risk, &'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: String,
    pub path: PathBuf,
    pub kind: Kind,
    pub rule: Rule,
    /// Engine `exclude` patterns (`~` form) that enabling this target adds to the config. Measurement skips
    /// them too, so size and risk describe only what the engine will compress.
    pub excludes: Vec<String>,
}

/// uv's cache holds cached ephemeral `uvx` environments (`archive-v0/<hash>/{bin,lib,...}`) that are APFS
/// clones of each other; compressing them would split the clones. Wheel entries there are private.
const UV_ENV_EXCLUDES: [&str; 2] = [
    "~/.cache/uv/archive-v0/*/lib/python*/**",
    "~/.cache/uv/archive-v0/*/bin/**",
];

fn entry(id: &str, path: PathBuf, kind: Kind, rule: Rule) -> Entry {
    Entry {
        id: id.into(),
        path,
        kind,
        rule,
        excludes: Vec::new(),
    }
}

pub fn builtin(home: &Path) -> Vec<Entry> {
    use Kind::*;
    vec![
        entry("rustup", home.join(".rustup"), Toolchain, Rule::Allowed),
        entry(
            "cargo-registry",
            home.join(".cargo/registry"),
            PackageCache,
            Rule::Allowed,
        ),
        entry("npm", home.join(".npm"), PackageCache, Rule::Allowed),
        entry(
            "mise",
            home.join(".local/share/mise"),
            Toolchain,
            Rule::Allowed,
        ),
        Entry {
            excludes: UV_ENV_EXCLUDES.map(String::from).to_vec(),
            ..entry(
                "uv-cache",
                home.join(".cache/uv"),
                PackageCache,
                Rule::VenvCloneScan,
            )
        },
        entry(
            "bun-store",
            home.join(".bun/install"),
            Store,
            Rule::Blocked(
                Risk::BlockedByRule,
                "bun installs by clonefile; compressing un-shares the clones",
            ),
        ),
        // pnpm's global virtual store uses symlinks; hardlinked files are skipped by the engine anyway.
        entry(
            "pnpm-store",
            home.join("Library/pnpm"),
            Store,
            Rule::Allowed,
        ),
        // Pseudo target: a glob, never measured.
        entry(
            "rust-target",
            home.join("**/target"),
            Build,
            Rule::Blocked(
                Risk::RewrittenOften,
                "rewritten on every build; some target dirs are deduped with fclones clones",
            ),
        ),
    ]
}

/// Entry for a path the catalog does not know. Rust `target` dirs stay blocked.
pub fn custom(path: PathBuf, home: &Path) -> Entry {
    let rule = if path.file_name().is_some_and(|n| n == "target") {
        Rule::Blocked(
            Risk::RewrittenOften,
            "rust build dir: rewritten on every build",
        )
    } else {
        Rule::Allowed
    };
    Entry {
        id: format!("custom:{}", contract_tilde(&path, home)),
        path,
        kind: Kind::Custom,
        rule,
        excludes: Vec::new(),
    }
}

/// Catalog plus one `custom` entry for every (already expanded) pattern not in the catalog.
pub fn with_custom(home: &Path, patterns: &[PathBuf]) -> Vec<Entry> {
    let mut all = builtin(home);
    for p in patterns {
        if !all.iter().any(|e| &e.path == p) {
            all.push(custom(p.clone(), home));
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_unknown_patterns_only() {
        let home = Path::new("/h");
        let all = with_custom(
            home,
            &[
                "/h/.npm".into(),
                "/h/proj/.venv".into(),
                "/h/proj/target".into(),
            ],
        );
        assert_eq!(all.len(), builtin(home).len() + 2);
        let venv = all.iter().find(|e| e.id == "custom:~/proj/.venv").unwrap();
        assert_eq!((venv.kind, &venv.rule), (Kind::Custom, &Rule::Allowed));
        let target = all.iter().find(|e| e.id == "custom:~/proj/target").unwrap();
        assert!(matches!(
            target.rule,
            Rule::Blocked(Risk::RewrittenOften, _)
        ));
    }
}
