//! Small helpers shared by the modules: home directory, `~` handling, RFC3339 clock.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Expands a leading `~` or `~/` to `home`.
pub fn expand_tilde(p: &str, home: &Path) -> PathBuf {
    match p {
        "~" => home.to_path_buf(),
        _ => p
            .strip_prefix("~/")
            .map_or_else(|| PathBuf::from(p), |rest| home.join(rest)),
    }
}

/// Inverse of `expand_tilde`: `/Users/x/.npm` -> `~/.npm` (paths outside `home` are unchanged).
pub fn contract_tilde(p: &Path, home: &Path) -> String {
    match p.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".into(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => p.display().to_string(),
    }
}

/// Current UTC time as RFC3339 (`2026-11-24T15:18:00Z`).
pub fn now_rfc3339() -> String {
    rfc3339(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    )
}

fn rfc3339(secs: u64) -> String {
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Inverse of `now_rfc3339` (`YYYY-MM-DDTHH:MM:SSZ`, UTC only) to Unix seconds.
pub fn rfc3339_to_secs(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    let shape = b.len() == 20
        && s.is_ascii()
        && b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':'
        && b[19] == b'Z';
    if !shape {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s[r].parse::<i64>().ok();
    let (y, m, d, hh, mm, ss) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh > 23 || mm > 59 || ss > 60 {
        return None;
    }
    let y = y - i64::from(m <= 2);
    let (era, yoe) = (y.div_euclid(400), y.rem_euclid(400));
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let days = era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468;
    u64::try_from(days * 86_400 + hh * 3600 + mm * 60 + ss).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_known_values() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(rfc3339(1_709_164_800), "2024-02-29T00:00:00Z");
    }

    #[test]
    fn rfc3339_round_trip() {
        for secs in [0, 1_700_000_000, 1_709_164_800, 1_795_545_367] {
            assert_eq!(rfc3339_to_secs(&rfc3339(secs)), Some(secs));
        }
        assert_eq!(rfc3339_to_secs("never"), None);
        assert_eq!(rfc3339_to_secs("2026-13-01T00:00:00Z"), None);
    }

    #[test]
    fn tilde_round_trip() {
        let home = Path::new("/Users/x");
        assert_eq!(expand_tilde("~/.npm", home), Path::new("/Users/x/.npm"));
        assert_eq!(expand_tilde("/abs", home), Path::new("/abs"));
        assert_eq!(contract_tilde(Path::new("/Users/x/.npm"), home), "~/.npm");
        assert_eq!(contract_tilde(Path::new("/opt/y"), home), "/opt/y");
    }
}
