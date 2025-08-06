use chrono::Local;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::{Receiver, channel},
    time::{Duration, Instant},
};
const PUSH_INTERVAL: Duration = Duration::from_secs(4 * 60  + 30); // 4.5 minutes

pub fn run_git_watcher(repo_path: String) {
    let log_file = setup_logger(&repo_path);
    log(&log_file, "Service started.");

    let (tx, rx) = channel();
    let _watcher = RecommendedWatcher::new(tx, Config::default())
        .and_then(|mut w| {
            w.watch(Path::new(&repo_path), RecursiveMode::Recursive)
                .map(|_| w)
        })
        .expect("Failed to set up watcher");

    event_loop(rx, repo_path, log_file);
}

fn event_loop(rx: Receiver<notify::Result<notify::Event>>, repo_path: String, log_file: PathBuf) {
    let mut last_change: Option<Instant> = None;

    loop {
        let timeout = match last_change {
            Some(time) => {
                let elapsed = time.elapsed();
                if elapsed >= PUSH_INTERVAL {
                    run_git(&repo_path, &log_file);
                    last_change = None;
                    continue;
                }
                PUSH_INTERVAL - elapsed
            }
            None => Duration::from_secs(u64::MAX),
        };

        match rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                ) {
                    let relevant = event.paths.iter().any(|path| {
                        path.strip_prefix(&repo_path)
                            .ok()
                            .map(|p| check_if_relevant(p))
                            .unwrap_or(false)
                    });

                    if relevant {
                        last_change = Some(Instant::now());
                        log(
                            &log_file,
                            &format!("Change: {:?} ({:?})", event.paths, event.kind),
                        );
                    }
                }
            }
            Ok(Err(e)) => {
                log(&log_file, &format!("Notify error: {:?}", e));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Timeout occurred, check if we should push
                if last_change.is_some() {
                    run_git(&repo_path, &log_file);
                    last_change = None;
                }
            }
            Err(_) => break, // channel closed
        }
    }

    log(&log_file, "Service shutting down normally.");
}

fn check_if_relevant(p: &Path) -> bool {
    let filename = p.file_name().and_then(|f| f.to_str()).unwrap_or("");

    if filename == "index.html" || filename == "contents.json" {
        return false;
    }

    for ancestor in p.ancestors() {
        if let Some(part) = ancestor.file_name().and_then(|s| s.to_str()) {
            if part == "logs" {
                return false;
            }
        }
    }

    true
}

fn setup_logger(repo_path: &str) -> PathBuf {
    let log_dir = Path::new(repo_path).join("logs");
    let _ = fs::create_dir_all(&log_dir);

    let date = Local::now().format("%Y-%m-%d").to_string();
    let log_path = log_dir.join(format!("{}.log", date));

    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Unable to open or create log file");

    log_path
}

fn log(log_file: &Path, message: &str) {
    let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();
    let entry = format!("{} {}\n", timestamp, message);

    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .and_then(|mut f| f.write_all(entry.as_bytes()));
}

fn run_git(repo_path: &str, log_file: &Path) {
    let script_path = std::env::var("CDN_REPO_SYNC_FILES_SCRIPT_PATH")
        .expect("CDN_REPO_SYNC_FILES_SCRIPT_PATH environment variable not set");

    let mut parts = script_path.splitn(2, ';');
    let python_path = parts.next().expect("Missing Python path").replace("\"", "");
    let script_path = parts
        .next()
        .expect("Missing script path")
        .replace("\"", "")
        .replace("\\", "/");

    println!("Running Python script: {}", script_path);

    let output = Command::new(python_path)
        .current_dir(repo_path)
        .arg(script_path)
        .output()
        .expect("Failed to execute Python script");
    if !output.status.success() {
        log(
            log_file,
            &format!("Python script failed with status: {}", output.status),
        );

        log(
            log_file,
            &format!(
                "Python script output: {}",
                String::from_utf8_lossy(&output.stdout)
            ),
        );
        log(
            log_file,
            &format!(
                "Python script error output: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        );
        return;
    }

    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let commit_msg = format!("auto: synchronize and update files at {}", now);

    let add_output = Command::new("git")
        .args(["-C", repo_path, "add", "."])
        .output();

    let commit_output = Command::new("git")
        .args(["-C", repo_path, "commit", "-m", &commit_msg])
        .output();

    let push_output = Command::new("git")
        .args(["-C", repo_path, "push", "origin", "master"])
        .output();

    if let Ok(out) = add_output {
        log(log_file, "--- git add ---");
        log_process_output(log_file, &out);
    }

    if let Ok(out) = commit_output {
        log(log_file, "--- git commit ---");
        log_process_output(log_file, &out);
    }

    if let Ok(out) = push_output {
        log(log_file, "--- git push ---");
        log_process_output(log_file, &out);
    }
}

fn log_process_output(log_file: &Path, output: &std::process::Output) {
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        log(log_file, &format!("{}", stdout.trim()));
    }
}
