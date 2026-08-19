use std::{fs, path::Path, process::Command};

use serie::git::{self, Repository};

type TestResult = Result<(), Box<dyn std::error::Error>>;

// The identity actually recorded in the commits.
const RAW_AUTHOR_NAME: &str = "Old Author";
const RAW_AUTHOR_EMAIL: &str = "old-author@example.com";
const RAW_COMMITTER_NAME: &str = "Old Committer";
const RAW_COMMITTER_EMAIL: &str = "old-committer@example.com";

// The canonical identity declared in .mailmap.
const MAPPED_AUTHOR_NAME: &str = "New Author";
const MAPPED_AUTHOR_EMAIL: &str = "new-author@example.com";
const MAPPED_COMMITTER_NAME: &str = "New Committer";
const MAPPED_COMMITTER_EMAIL: &str = "new-committer@example.com";

#[test]
fn mailmap_enabled_rewrites_author_and_committer() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();
    let git = TestGit::new(repo_path);

    git.init();
    git.commit("commit");
    write_mailmap(repo_path);

    let repository = Repository::load(repo_path, git::SortCommit::Chronological, None, true)?;
    let commits = repository.all_commits();
    let commit = commits.first().unwrap();

    assert_eq!(commit.author_name, MAPPED_AUTHOR_NAME);
    assert_eq!(commit.author_email, MAPPED_AUTHOR_EMAIL);
    assert_eq!(commit.committer_name, MAPPED_COMMITTER_NAME);
    assert_eq!(commit.committer_email, MAPPED_COMMITTER_EMAIL);

    Ok(())
}

#[test]
fn mailmap_disabled_keeps_raw_identity() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();
    let git = TestGit::new(repo_path);

    git.init();
    git.commit("commit");
    write_mailmap(repo_path);

    let repository = Repository::load(repo_path, git::SortCommit::Chronological, None, false)?;
    let commits = repository.all_commits();
    let commit = commits.first().unwrap();

    assert_eq!(commit.author_name, RAW_AUTHOR_NAME);
    assert_eq!(commit.author_email, RAW_AUTHOR_EMAIL);
    assert_eq!(commit.committer_name, RAW_COMMITTER_NAME);
    assert_eq!(commit.committer_email, RAW_COMMITTER_EMAIL);

    Ok(())
}

#[test]
fn mailmap_enabled_without_mailmap_file_is_a_no_op() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();
    let git = TestGit::new(repo_path);

    git.init();
    git.commit("commit");

    let repository = Repository::load(repo_path, git::SortCommit::Chronological, None, true)?;
    let commits = repository.all_commits();
    let commit = commits.first().unwrap();

    assert_eq!(commit.author_name, RAW_AUTHOR_NAME);
    assert_eq!(commit.author_email, RAW_AUTHOR_EMAIL);
    assert_eq!(commit.committer_name, RAW_COMMITTER_NAME);
    assert_eq!(commit.committer_email, RAW_COMMITTER_EMAIL);

    Ok(())
}

fn write_mailmap(repo_path: &Path) {
    let content = format!(
        "{MAPPED_AUTHOR_NAME} <{MAPPED_AUTHOR_EMAIL}> {RAW_AUTHOR_NAME} <{RAW_AUTHOR_EMAIL}>\n\
         {MAPPED_COMMITTER_NAME} <{MAPPED_COMMITTER_EMAIL}> {RAW_COMMITTER_NAME} <{RAW_COMMITTER_EMAIL}>\n"
    );
    fs::write(repo_path.join(".mailmap"), content).unwrap();
}

struct TestGit<'a> {
    path: &'a Path,
}

impl TestGit<'_> {
    fn new(path: &Path) -> TestGit<'_> {
        TestGit { path }
    }

    fn init(&self) {
        self.run(&["init", "-b", "master"]);
    }

    fn commit(&self, message: &str) {
        self.run(&["commit", "--allow-empty", "-m", message]);
    }

    fn run(&self, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(self.path)
            .env("GIT_AUTHOR_NAME", RAW_AUTHOR_NAME)
            .env("GIT_AUTHOR_EMAIL", RAW_AUTHOR_EMAIL)
            .env("GIT_AUTHOR_DATE", "2024-01-01T01:02:03+00:00")
            .env("GIT_COMMITTER_NAME", RAW_COMMITTER_NAME)
            .env("GIT_COMMITTER_EMAIL", RAW_COMMITTER_EMAIL)
            .env("GIT_COMMITTER_DATE", "2024-01-01T01:02:03+00:00")
            .env("GIT_CONFIG_NOSYSTEM", "true")
            .env("HOME", "/dev/null")
            .status()
            .unwrap_or_else(|_| panic!("failed to execute git {}", args.join(" ")));
        assert!(status.success(), "git {} failed", args.join(" "));
    }
}
