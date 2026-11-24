use super::*;

const START: &str = "2026-11-24T16:54:46.056-03:00 INFO start auto-compress-apfs 0.4.2 config=/x threads=12 dry_run=false";
const FILE: &str =
    "2026-11-24T16:54:46.148-03:00 INFO compressed /tmp/my dir/f1.txt 400000 -> 4096";

fn tracker() -> Tracker {
    Tracker::new(Some(3), Duration::ZERO)
}

#[test]
fn parses_compressed_lines_with_spaces_in_the_path() {
    assert_eq!(
        parse_compressed(FILE),
        Some(("/tmp/my dir/f1.txt".into(), 400_000))
    );
    assert_eq!(parse_compressed("INFO summary found=3"), None);
    assert_eq!(parse_compressed("INFO compressed /x notanumber -> 1"), None);
}

#[test]
fn counts_files_and_bytes() {
    let mut t = tracker();
    assert_eq!(t.on_line(START).unwrap().phase, Phase::Starting);
    let p = t.on_line(FILE).unwrap();
    assert_eq!(
        (p.phase, p.files, p.total_files, p.bytes),
        (Phase::Compressing, Some(1), Some(3), Some(400_000))
    );
    assert_eq!(p.current_path.as_deref(), Some("/tmp/my dir/f1.txt"));
    assert_eq!(t.on_line(FILE).unwrap().files, Some(2));
    assert!(t.on_line("2026 INFO summary found=3").is_none());
}

#[test]
fn verify_lines_switch_phase_and_keep_counts() {
    let mut t = tracker();
    t.on_line(FILE);
    let p = t.on_line("2026 INFO verifying 1 files").unwrap();
    assert_eq!((p.phase, p.files), (Phase::Verifying, Some(1)));
}

#[test]
fn throttles_same_phase_updates_but_not_phase_changes() {
    let mut t = Tracker::new(None, Duration::from_secs(60));
    assert!(t.on_line(FILE).is_some());
    assert!(t.on_line(FILE).is_none()); // throttled, but still counted
    let p = t.on_line("INFO verifying").unwrap();
    assert_eq!(
        (p.phase, p.files, p.total_files),
        (Phase::Verifying, Some(2), None)
    );
}
