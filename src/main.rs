use std::{
    fs::{self, create_dir_all, read_dir},
    path::PathBuf,
    process::{Command, Stdio},
};

use clap::Parser;

/// Tool to search for code repositories and delete clean ones.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to recurse through to find repositories.
    root: PathBuf,

    /// Directory to store detailed results in.
    results_dir: PathBuf,
}

#[derive(Clone)]
struct GitRepo {
    root: PathBuf,
}

fn git_cmd(args: &[&str], repo: &PathBuf) -> String {
    let child = Command::new("git")
        .current_dir(repo)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("git failed to start")
        .wait_with_output()
        .unwrap();

    assert!(child.status.success(), "git command failed");

    str::from_utf8(&child.stdout).unwrap().into()
}

impl GitRepo {
    fn is_dirty(&self) -> bool {
        // Check for a dirty git tree (untracked, unstaged, or staged changed).
        let stdout = git_cmd(&["status", "--porcelain"], &self.root);
        !stdout.is_empty() // Some output means the tree is dirty.
    }

    fn has_unpushed_branches(&self) -> bool {
        // Check for unpushed branches.
        let stdout = git_cmd(&["log", "--branches", "--not", "--remotes"], &self.root);
        !stdout.is_empty() // Some output means there are unpushed branches.
    }
}

struct RepoSearchResults {
    repos: Vec<GitRepo>,
    misc_files: Vec<PathBuf>,
}

impl RepoSearchResults {
    fn search(root: PathBuf) -> Self {
        let mut repos = Vec::new();
        let mut misc_files = Vec::new();

        let mut fringe = vec![root];
        while let Some(p) = fringe.pop() {
            let is_repo = p.join(".git").exists();
            if is_repo {
                repos.push(GitRepo { root: p });
                continue;
            }

            for child in read_dir(p.clone()).unwrap() {
                let child_path = child.unwrap().path();
                if child_path.is_dir() {
                    fringe.push(child_path);
                } else {
                    misc_files.push(child_path);
                }
            }
        }

        Self { repos, misc_files }
    }
}

fn main() {
    let args = Args::parse();

    eprint!("Trawling through {}...", args.root.to_string_lossy());
    let search_results = RepoSearchResults::search(args.root);
    let pretty_count = pluralize(search_results.repos.len(), "repo");
    eprintln!(" found {pretty_count}!");

    eprintln!("Analyzing repos...");
    let mut synced_repos: Vec<GitRepo> = Vec::new();
    let mut dirty_repos: Vec<GitRepo> = Vec::new();
    let mut unpushed_repos: Vec<GitRepo> = Vec::new();
    for repo in tqdm::tqdm(search_results.repos) {
        let mut synced = true;
        if repo.is_dirty() {
            synced = false;
            dirty_repos.push(repo.clone());
        }

        if repo.has_unpushed_branches() {
            synced = false;
            unpushed_repos.push(repo.clone());
        }

        if synced {
            synced_repos.push(repo);
        }
    }

    fn dump_result<'a, I>(results_dir: &PathBuf, filename: &str, files: I) -> String
    where
        I: Iterator<Item = &'a PathBuf>,
    {
        if !results_dir.exists() {
            create_dir_all(results_dir).unwrap();
        }
        let result_file = results_dir.join(filename);
        let contents = files
            .map(|f: &PathBuf| f.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join("\n");
        fs::write(&result_file, contents).unwrap();

        format!(" (see {})", result_file.to_string_lossy())
    }

    eprintln!("\n# Summary");

    let pretty_count = pluralize(synced_repos.len(), "synced repo");
    eprint!("Found {pretty_count}");
    eprintln!(
        "{}",
        dump_result(
            &args.results_dir,
            "synced.txt",
            synced_repos.iter().map(|r| &r.root),
        )
    );

    let pretty_count = pluralize(dirty_repos.len(), "dirty repo");
    eprint!("Found {pretty_count}");
    eprintln!(
        "{}",
        dump_result(
            &args.results_dir,
            "dirty.txt",
            dirty_repos.iter().map(|r| &r.root),
        )
    );

    let pretty_count = pluralize(unpushed_repos.len(), "unpushed repo");
    eprint!("Found {pretty_count}");
    eprintln!(
        "{}",
        dump_result(
            &args.results_dir,
            "unpushed.txt",
            unpushed_repos.iter().map(|r| &r.root),
        )
    );

    let pretty_count = pluralize(search_results.misc_files.len(), "misc file");
    eprint!("Found {pretty_count} not inside of a repo");
    eprintln!(
        "{}",
        dump_result(
            &args.results_dir,
            "misc.txt",
            search_results.misc_files.iter(),
        )
    );
}

fn pluralize(count: usize, thing: &str) -> String {
    let pluralize = count == 0 || count > 1;
    let mut result = format!("{count} {thing}");
    if pluralize {
        result += "s";
    }

    result
}
