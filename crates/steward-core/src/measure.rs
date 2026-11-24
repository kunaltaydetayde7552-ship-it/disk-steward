//! Clone-aware size measurement. `du` counts clones twice; `private_bytes` (getattrlist
//! ATTR_CMNEXT_PRIVATESIZE) only counts bytes not shared with any other file.

use crate::model::Measure;
use globset::GlobSet;
use std::{
    collections::HashSet,
    ffi::CString,
    fs, io,
    os::{
        macos::fs::MetadataExt as _,
        unix::{ffi::OsStrExt, fs::MetadataExt},
    },
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

const WORKERS: usize = 4;

const UF_COMPRESSED: u32 = 0x20; // sys/stat.h
// Header values (sys/attr.h): ATTR_CMNEXT_PRIVATESIZE 0x8 (in `forkattr`), FSOPT_ATTR_CMN_EXTENDED 0x20.
const OPTIONS: u32 = libc::FSOPT_NOFOLLOW | libc::FSOPT_ATTR_CMN_EXTENDED;

/// Walks `root` with a small worker pool (symlinks are not followed, other filesystems are not entered).
/// `private_bytes` is `None` when the filesystem refuses the attribute for any file.
pub fn measure(root: &Path) -> io::Result<Measure> {
    measure_excluding(root, &GlobSet::empty())
}

/// Like `measure`, skipping files whose full path matches `skip` (engine `exclude` semantics).
pub fn measure_excluding(root: &Path, skip: &GlobSet) -> io::Result<Measure> {
    let root_md = fs::metadata(root)?;
    let seen = Mutex::new(HashSet::new());
    if root_md.is_file() {
        let mut acc = Acc::new(&seen);
        acc.file(root, &root_md);
        return Ok(acc.m);
    }
    let dev = root_md.dev();
    let queue = Mutex::new(vec![root.to_path_buf()]);
    let pending = AtomicUsize::new(1); // directories queued or being read
    Ok(thread::scope(|s| {
        let workers: Vec<_> = (0..WORKERS)
            .map(|_| s.spawn(|| walk(dev, &queue, &pending, &seen, skip)))
            .collect();
        workers
            .into_iter()
            .map(|w| w.join().expect("measure worker panicked"))
            .fold(Acc::new(&seen).m, merge)
    }))
}

fn walk(
    dev: u64,
    queue: &Mutex<Vec<PathBuf>>,
    pending: &AtomicUsize,
    seen: &Mutex<HashSet<(u64, u64)>>,
    skip: &GlobSet,
) -> Measure {
    let mut acc = Acc::new(seen);
    loop {
        let next = queue.lock().expect("queue lock").pop();
        let Some(dir) = next else {
            if pending.load(Ordering::SeqCst) == 0 {
                return acc.m;
            }
            thread::sleep(Duration::from_micros(200));
            continue;
        };
        let mut subdirs = Vec::new();
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let Ok(md) = entry.metadata() else { continue }; // lstat: never follows symlinks
            if md.dev() != dev {
                continue;
            }
            if md.is_dir() {
                subdirs.push(entry.path());
            } else if md.is_file() && !skip.is_match(entry.path()) {
                acc.file(&entry.path(), &md);
            }
        }
        pending.fetch_add(subdirs.len(), Ordering::SeqCst); // before the parent is released
        queue.lock().expect("queue lock").extend(subdirs);
        pending.fetch_sub(1, Ordering::SeqCst);
    }
}

fn merge(a: Measure, b: Measure) -> Measure {
    Measure {
        logical_bytes: a.logical_bytes + b.logical_bytes,
        on_disk_bytes: a.on_disk_bytes + b.on_disk_bytes,
        private_bytes: a.private_bytes.zip(b.private_bytes).map(|(x, y)| x + y),
        total_files: a.total_files + b.total_files,
        compressed_files: a.compressed_files + b.compressed_files,
    }
}

struct Acc<'a> {
    m: Measure,
    seen: &'a Mutex<HashSet<(u64, u64)>>,
}

impl<'a> Acc<'a> {
    fn new(seen: &'a Mutex<HashSet<(u64, u64)>>) -> Self {
        Self {
            m: Measure {
                private_bytes: Some(0),
                ..Measure::default()
            },
            seen,
        }
    }

    fn file(&mut self, path: &Path, md: &fs::Metadata) {
        if md.nlink() > 1
            && !self
                .seen
                .lock()
                .expect("seen lock")
                .insert((md.dev(), md.ino()))
        {
            return; // hardlink already counted
        }
        let on_disk = md.blocks() * 512;
        let compressed = md.st_flags() & UF_COMPRESSED != 0;
        self.m.total_files += 1;
        self.m.logical_bytes += md.len();
        self.m.on_disk_bytes += on_disk;
        self.m.compressed_files += u64::from(compressed);
        // The kernel reports 0 private bytes for compressed files (their data sits in the resource
        // fork / decmpfs xattr, not in the data fork). The engine writes compressed copies fresh
        // (never clones), so their on-disk size is private.
        let private = if compressed {
            Ok(on_disk)
        } else {
            private_size(path)
        };
        match private {
            Ok(n) => self.m.private_bytes = self.m.private_bytes.map(|p| p + n),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {} // vanished mid-walk
            Err(_) => self.m.private_bytes = None,
        }
    }
}

/// Bytes of `path` not shared with any clone.
pub fn private_size(path: &Path) -> io::Result<u64> {
    let c = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    // SAFETY: attrlist is plain old data; zeroed is a valid initial value.
    let mut list: libc::attrlist = unsafe { std::mem::zeroed() };
    list.bitmapcount = libc::ATTR_BIT_MAP_COUNT;
    list.forkattr = libc::ATTR_CMNEXT_PRIVATESIZE;
    // Reply layout: u32 total length, then off_t (read bytewise: packing is not 8-aligned).
    let mut buf = [0u8; 12];
    // SAFETY: valid path, attrlist and buffer of the size passed.
    let rc = unsafe {
        libc::getattrlist(
            c.as_ptr(),
            (&raw mut list).cast(),
            buf.as_mut_ptr().cast(),
            buf.len(),
            OPTIONS,
        )
    };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    let size = i64::from_ne_bytes(buf[4..12].try_into().expect("8 bytes"));
    u64::try_from(size)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "negative private size"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn constants_match_sdk_header() {
        assert_eq!(libc::ATTR_CMNEXT_PRIVATESIZE, 0x8);
        assert_eq!(libc::FSOPT_ATTR_CMN_EXTENDED, 0x20);
        assert_eq!(libc::FSOPT_NOFOLLOW, 0x1);
        assert_eq!(libc::ATTR_BIT_MAP_COUNT, 5);
    }

    #[test]
    fn counts_files_and_skips_symlinks() {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir(t.path().join("d")).unwrap();
        fs::write(t.path().join("d/a"), vec![1u8; 10_000]).unwrap();
        fs::write(t.path().join("b"), vec![2u8; 5_000]).unwrap();
        std::os::unix::fs::symlink(t.path().join("d"), t.path().join("link")).unwrap();
        fs::hard_link(t.path().join("b"), t.path().join("b2")).unwrap();
        let m = measure(t.path()).unwrap();
        assert_eq!(
            (m.total_files, m.logical_bytes, m.compressed_files),
            (2, 15_000, 0)
        );
        assert!(m.on_disk_bytes >= 15_000);
    }

    #[test]
    fn excluded_paths_are_not_measured() {
        let t = tempfile::tempdir().unwrap();
        let env = t.path().join("archive-v0/h1/lib/python3.13");
        fs::create_dir_all(&env).unwrap();
        fs::create_dir_all(t.path().join("archive-v0/wheel")).unwrap();
        let data: Vec<u8> = (0..4_194_304u32)
            .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
            .collect();
        fs::write(env.join("orig"), &data).unwrap();
        let cloned = Command::new("cp")
            .arg("-c")
            .arg(env.join("orig"))
            .arg(env.join("clone"))
            .status()
            .unwrap();
        if !cloned.success() {
            eprintln!("SKIP: cp -c failed (temp dir is not on APFS?)");
            return;
        }
        fs::write(t.path().join("archive-v0/wheel/mod.py"), "x = 1").unwrap();
        let all = measure(t.path()).unwrap();
        assert!(
            all.private_bytes.unwrap() < all.on_disk_bytes,
            "clones must show as shared"
        );
        let mut skip = globset::GlobSetBuilder::new();
        let pat = format!("{}/archive-v0/*/lib/python*/**", t.path().display());
        skip.add(
            globset::GlobBuilder::new(&pat)
                .literal_separator(true)
                .build()
                .unwrap(),
        );
        let m = measure_excluding(t.path(), &skip.build().unwrap()).unwrap();
        assert_eq!(m.total_files, 1);
        assert_eq!(m.private_bytes, Some(m.on_disk_bytes));
    }

    #[test]
    fn clone_has_smaller_private_size_than_on_disk() {
        let t = tempfile::tempdir().unwrap();
        // Incompressible-ish content so allocation is plain 4 MiB.
        let data: Vec<u8> = (0..4_194_304u32)
            .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
            .collect();
        fs::write(t.path().join("orig"), &data).unwrap();
        let cloned = Command::new("cp")
            .arg("-c")
            .arg(t.path().join("orig"))
            .arg(t.path().join("clone"))
            .status()
            .unwrap();
        if !cloned.success() {
            eprintln!("SKIP: cp -c failed (temp dir is not on APFS?)");
            return;
        }
        let m = measure(t.path()).unwrap();
        let private = m.private_bytes.expect("private size supported");
        assert!(
            private < m.on_disk_bytes,
            "private {private} vs on disk {}",
            m.on_disk_bytes
        );
        // After un-sharing, a plain copy is fully private.
        fs::write(
            t.path().join("unique"),
            data.iter().rev().copied().collect::<Vec<_>>(),
        )
        .unwrap();
        let alone = measure(&t.path().join("unique")).unwrap();
        assert_eq!(alone.private_bytes, Some(alone.on_disk_bytes));
    }
}
