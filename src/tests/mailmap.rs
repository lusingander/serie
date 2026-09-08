use std::{fs, path::Path};

use crate::{
    git::{self, Repository},
    test_git::GitRepository,
};

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
    let git = test_git(repo_path);

    git.init();
    git.commit("commit", "2024-01-01");
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
    let git = test_git(repo_path);

    git.init();
    git.commit("commit", "2024-01-01");
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
    let git = test_git(repo_path);

    git.init();
    git.commit("commit", "2024-01-01");

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

fn test_git(path: &Path) -> GitRepository<'_> {
    GitRepository::new(path).with_identities(
        RAW_AUTHOR_NAME,
        RAW_AUTHOR_EMAIL,
        RAW_COMMITTER_NAME,
        RAW_COMMITTER_EMAIL,
    )
}
