use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, SystemTime},
};

use crate::event::{AppEvent, Sender};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
// Wait until the repository has been quiet for this long before refreshing.
const DEBOUNCE: Duration = Duration::from_millis(500);

type Fingerprint = Vec<(PathBuf, Option<(SystemTime, u64)>)>;

/// Starts a background thread that notifies the app when `HEAD`, `refs/`
/// or `packed-refs` of the repository change.
pub fn start(repo_path: &Path, tx: Sender) {
    let dirs = git_dirs(repo_path);
    if dirs.is_empty() {
        return;
    }
    thread::spawn(move || {
        let mut last = fingerprint(&dirs);
        loop {
            thread::sleep(POLL_INTERVAL);
            let current = fingerprint(&dirs);
            if current == last {
                continue;
            }
            // debounce: wait for changes to settle
            let mut settled = current;
            loop {
                thread::sleep(DEBOUNCE);
                let next = fingerprint(&dirs);
                if next == settled {
                    break;
                }
                settled = next;
            }
            last = settled;
            tx.send(AppEvent::RepositoryChanged);
        }
    });
}

fn git_dirs(repo_path: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for arg in ["--absolute-git-dir", "--git-common-dir"] {
        let Ok(output) = Command::new("git")
            .arg("rev-parse")
            .arg(arg)
            .current_dir(repo_path)
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let p = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
        let p = if p.is_absolute() {
            p
        } else {
            repo_path.join(p)
        };
        if !dirs.contains(&p) {
            dirs.push(p);
        }
    }
    dirs
}

fn fingerprint(dirs: &[PathBuf]) -> Fingerprint {
    let mut entries = Vec::new();
    for dir in dirs {
        stat(&dir.join("HEAD"), &mut entries);
        stat(&dir.join("packed-refs"), &mut entries);
        walk(&dir.join("refs"), &mut entries);
    }
    entries
}

fn stat(path: &Path, entries: &mut Fingerprint) {
    let meta = fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok().map(|t| (t, m.len())));
    entries.push((path.to_path_buf(), meta));
}

fn walk(dir: &Path, entries: &mut Fingerprint) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<_> = read_dir.flatten().map(|e| e.path()).collect();
    children.sort();
    for path in children {
        if path.is_dir() {
            walk(&path, entries);
        } else {
            stat(&path, entries);
        }
    }
}
