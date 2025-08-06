use watcher_logic::run_git_watcher;

fn main() {
    let repo_path =
        std::env::var("CDN_REPO_PATH");

    if repo_path.is_err() {
        eprintln!("CDN_REPO_PATH environment variable not set");
        std::process::exit(1);
    
    }

    run_git_watcher(repo_path.unwrap());
}
