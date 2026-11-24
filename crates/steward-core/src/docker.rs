//! Optional `docker system df` summary; `available: false` when the daemon is not reachable.

use crate::{model::DockerUsage, util::home_dir};
use std::process::Command;

/// A GUI-launched app has a minimal PATH, so well-known install locations are tried as well.
fn candidates() -> Vec<String> {
    let orb = home_dir()
        .join(".orbstack/bin/docker")
        .display()
        .to_string();
    [
        "docker",
        "/usr/local/bin/docker",
        "/opt/homebrew/bin/docker",
    ]
    .map(String::from)
    .into_iter()
    .chain([orb])
    .collect()
}

pub fn usage() -> DockerUsage {
    for bin in candidates() {
        if let Ok(out) = Command::new(bin)
            .args(["system", "df", "--format", "json"])
            .output()
            && out.status.success()
        {
            return parse(&String::from_utf8_lossy(&out.stdout));
        }
    }
    DockerUsage::default()
}

/// Parses the JSON lines of `docker system df --format json` (decimal units, e.g. `23.27GB`).
pub fn parse(text: &str) -> DockerUsage {
    let mut u = DockerUsage {
        available: true,
        ..DockerUsage::default()
    };
    for row in text
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
    {
        let field = |k: &str| {
            row.get(k)
                .and_then(|v| v.as_str())
                .and_then(parse_size)
                .unwrap_or(0)
        };
        u.reclaimable_bytes += field("Reclaimable");
        match row.get("Type").and_then(|t| t.as_str()) {
            Some("Images") => u.images_bytes = field("Size"),
            Some("Build Cache") => u.build_cache_bytes = field("Size"),
            Some("Local Volumes") => u.volumes_bytes = field("Size"),
            _ => {}
        }
    }
    u
}

/// `"1.72GB (7%)"` -> bytes (docker uses decimal units: kB, MB, GB, TB).
fn parse_size(text: &str) -> Option<u64> {
    let word = text.split_whitespace().next()?;
    let at = word.find(|c: char| c.is_ascii_alphabetic())?;
    let mult = match &word[at..] {
        "B" => 1.0,
        "kB" => 1e3,
        "MB" => 1e6,
        "GB" => 1e9,
        "TB" => 1e12,
        _ => return None,
    };
    Some((word[..at].parse::<f64>().ok()? * mult).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_df_fixture() {
        let u = parse(include_str!("../tests/fixtures/docker_df.jsonl"));
        assert!(u.available);
        assert_eq!(
            (u.images_bytes, u.build_cache_bytes, u.volumes_bytes),
            (23_270_000_000, 8_684_000_000, 22_880_000_000)
        );
        // 1.72 + 1.952 + 2.714 + 8.684 GB
        assert_eq!(u.reclaimable_bytes, 15_070_000_000);
    }

    #[test]
    fn size_units() {
        assert_eq!(parse_size("0B"), Some(0));
        assert_eq!(parse_size("1.5kB"), Some(1500));
        assert_eq!(parse_size("1.72GB (7%)"), Some(1_720_000_000));
        assert_eq!(parse_size("12 parsecs"), None);
    }
}
