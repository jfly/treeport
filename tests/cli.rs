use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    io::Cursor,
    path::{Path, PathBuf},
};

use assert_cmd::{Command, cargo::cargo_bin_cmd};
use predicates::prelude::*;

#[test]
fn root_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("treeport");

    cmd.arg("examples/git-treeport.toml")
        .arg("--root")
        .arg("/i/do/not/exit");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Error: Roots must exist and be directories.",
    ));

    Ok(())
}

struct GitRepo {
    root: PathBuf,
}

impl GitRepo {
    fn new(root: &Path) -> Self {
        Self { root: root.into() }
    }

    fn git(&self, args: &[&str]) {
        Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .assert()
            .success();
    }

    fn init(self) -> Self {
        self._init(false);
        self
    }

    fn init_bare(self) -> Self {
        self._init(true);
        self
    }

    fn _init(&self, bare: bool) {
        assert!(!self.root.exists());
        create_dir_all(&self.root).unwrap();
        if bare {
            self.git(&["init", "--bare"]);
        } else {
            self.git(&["init"]);
        };
    }

    fn create_file(&self, filename: &str, contents: &str) {
        fs::write(self.root.join(filename), contents).unwrap();
        self.git(&["add", "--intent-to-add", filename]);
    }

    fn add_remote(&self, name: &str, remote: &str) {
        self.git(&["remote", "add", name, remote]);
    }

    fn commit_all(&self, msg: &str) {
        self.git(&["commit", "--all", "--message", msg]);
    }

    fn push(&self) {
        self.git(&["push", "--set-upstream", "origin", "main"]);
    }
}

#[test]
fn simple_report() -> Result<(), Box<dyn std::error::Error>> {
    // Begin test setup.
    let temp_dir = tempfile::tempdir().unwrap();
    let tmp = temp_dir.path();

    let dirty_repo = GitRepo::new(&tmp.join("dirty-repo")).init();
    dirty_repo.create_file("README.md", "Dirty repo");

    let unpushed_repo = GitRepo::new(&tmp.join("unpushed-repo")).init();
    unpushed_repo.create_file("README.md", "Unpushed repo");
    unpushed_repo.commit_all("First commit");

    let origin_repo = GitRepo::new(&tmp.join("origin-repo")).init_bare();

    let synced_repo = GitRepo::new(&tmp.join("synced-repo")).init();
    synced_repo.create_file("README.md", "Synced repo");
    synced_repo.commit_all("First commit");
    synced_repo.add_remote("origin", &origin_repo.root.to_string_lossy());
    synced_repo.push();
    // End test setup.

    let mut cmd = cargo_bin_cmd!("treeport");
    cmd.arg("examples/git-treeport.toml").arg("--root").arg(tmp);
    let assert = cmd.assert().success();
    let output = assert.get_output();

    let mut record_by_path: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut reader = csv::Reader::from_reader(Cursor::new(&output.stdout));
    let headers = reader.headers().unwrap().clone();
    for record in reader.records() {
        let mut record: HashMap<String, String> = record
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, value)| (headers.get(i).unwrap().to_string(), value.to_string()))
            .collect();
        let path = PathBuf::from(record.get("path").unwrap());
        let rel_path = path.strip_prefix(tmp).unwrap().to_string_lossy();
        record.insert("path".to_string(), rel_path.to_string());
        record_by_path.insert(rel_path.to_string(), record);
    }

    let record = record_by_path.remove("dirty-repo").unwrap();
    assert_eq!(record.get("category").unwrap(), "git");
    assert_eq!(record.get("status").unwrap(), "dirty");

    let record = record_by_path.remove("unpushed-repo").unwrap();
    assert_eq!(record.get("category").unwrap(), "git");
    assert_eq!(record.get("status").unwrap(), "unpushed");

    let record = record_by_path.remove("origin-repo").unwrap();
    assert_eq!(record.get("category").unwrap(), "git-bare");

    let record = record_by_path.remove("synced-repo").unwrap();
    assert_eq!(record.get("category").unwrap(), "git");
    assert_eq!(record.get("status").unwrap(), "synced");

    assert_eq!(record_by_path, HashMap::new());

    Ok(())
}
