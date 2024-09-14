use std::{path::Path, process::Command};

use chrono::{DateTime, Days, NaiveDate, TimeZone, Utc};
use image::{GenericImage, GenericImageView};
use serie::{color, config::GraphColorConfig, git, graph};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const OUTPUT_DIR: &str = "./out/graph";
const SNAPSHOT_DIR: &str = "./tests/graph";

#[test]
fn straight_001() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    let mut base_date = Utc.with_ymd_and_hms(2024, 1, 1, 1, 2, 3).unwrap();
    for i in 1..=100 {
        let msg = &format!("{:03}", i);
        let date = &base_date.format("%Y-%m-%d").to_string();
        git.commit(msg, date);
        base_date = base_date.checked_add_days(Days::new(1)).unwrap();
    }

    git.log();

    let options = &[GenerateGraphOption::new(
        "straight_001",
        graph::SortCommit::Chronological,
    )];

    copy_git_dir(repo_path, "straight_001");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn branch_001() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.checkout("master");
    git.checkout_b("10");
    git.commit("011", "2024-02-01");

    git.checkout("master");
    git.checkout_b("20");
    git.commit("021", "2024-02-02");

    git.checkout("master");
    git.checkout_b("30");
    git.commit("031", "2024-02-03");

    git.checkout("master");
    git.checkout_b("40");
    git.commit("041", "2024-02-04");

    git.checkout("master");
    git.checkout_b("50");
    git.commit("051", "2024-02-05");

    git.checkout("10");
    git.commit("012", "2024-02-06");

    git.checkout("20");
    git.commit("022", "2024-02-07");

    git.checkout("30");
    git.commit("032", "2024-02-08");

    git.checkout("40");
    git.commit("042", "2024-02-09");

    git.checkout("50");
    git.commit("052", "2024-02-10");

    git.checkout("master");
    git.merge(&["10"], "2024-03-01");
    git.merge(&["20"], "2024-03-02");
    git.merge(&["30"], "2024-03-03");
    git.merge(&["40"], "2024-03-04");
    git.merge(&["50"], "2024-03-05");

    git.log();

    let options = &[
        GenerateGraphOption::new("branch_001_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("branch_001_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "branch_001");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn branch_002() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout("master");
    git.checkout_b("10");
    git.commit("011", "2024-02-01");

    git.checkout_b("20");
    git.commit("021", "2024-02-02");

    git.checkout_b("30");
    git.commit("031", "2024-02-03");

    git.checkout("10");
    git.commit("012", "2024-02-04");

    git.checkout("20");
    git.commit("022", "2024-02-05");

    git.checkout("10");
    git.checkout_b("40");
    git.commit("041", "2024-02-06");

    git.checkout("20");
    git.checkout_b("50");
    git.commit("51", "2024-02-07");

    git.checkout("30");
    git.commit("032", "2024-02-08");

    git.checkout("master");
    git.merge(&["40"], "2024-03-01");

    git.checkout("20");
    git.commit("023", "2024-03-02");

    git.checkout("master");
    git.merge(&["20"], "2024-03-03");

    git.checkout("10");
    git.commit("013", "2024-03-04");

    git.checkout("master");
    git.merge(&["10"], "2024-03-05");

    git.log();

    let options = &[
        GenerateGraphOption::new("branch_002_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("branch_002_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "branch_002");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn branch_003() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10");
    git.checkout_b("20");
    git.checkout_b("30");

    git.checkout("master");
    git.commit("002", "2024-01-02");

    git.checkout("10");
    git.commit("011", "2024-02-01");
    git.commit("012", "2024-02-02");

    git.checkout("20");
    git.commit("021", "2024-02-03");

    git.checkout("30");
    git.commit("031", "2024-02-04");

    git.checkout("20");
    git.commit("022", "2024-02-05");

    git.log();

    let options = &[
        GenerateGraphOption::new("branch_003_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("branch_003_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "branch_003");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn branch_004() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10");
    git.commit("011", "2024-02-01");

    git.checkout("master");
    git.merge(&["10"], "2024-02-02");

    git.checkout_b("20");
    git.commit("021", "2024-02-03");

    git.checkout("master");
    git.merge(&["20"], "2024-02-04");

    git.commit("002", "2024-02-05");

    git.checkout_b("30");
    git.checkout_b("40");
    git.checkout_b("50");

    git.checkout("30");
    git.commit("031", "2024-03-01");

    git.checkout("40");
    git.commit("041", "2024-03-02");

    git.checkout("50");
    git.commit("051", "2024-03-03");

    git.checkout("master");
    git.merge(&["40"], "2024-03-04");

    git.checkout_b("60");
    git.commit("061", "2024-04-01");

    git.checkout("50");
    git.commit("052", "2024-04-02");

    git.checkout("30");
    git.commit("032", "2024-04-03");

    git.checkout("master");
    git.commit("003", "2024-04-04");

    git.merge(&["30"], "2024-04-05");
    git.merge(&["50"], "2024-04-06");
    git.merge(&["60"], "2024-04-07");

    git.checkout_b("70");
    git.commit("071", "2024-05-01");

    git.checkout_b("80");
    git.commit("081", "2024-05-02");

    git.checkout("master");
    git.commit("004", "2024-05-03");

    git.log();

    let options = &[
        GenerateGraphOption::new("branch_004_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("branch_004_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "branch_004");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn branch_005() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10");

    git.checkout("master");
    git.commit("002", "2024-01-02");

    git.checkout("10");
    git.commit("011", "2024-02-01");

    git.checkout("master");
    git.commit("003", "2024-02-02");

    git.checkout_b("20");

    git.checkout("master");
    git.merge(&["10"], "2024-03-01");

    git.checkout("20");
    git.commit("021", "2024-03-02");

    git.checkout("master");
    git.merge(&["20"], "2024-03-03");

    git.log();

    let options = &[
        GenerateGraphOption::new("branch_005_chrono", serie::graph::SortCommit::Chronological),
        GenerateGraphOption::new("branch_005_topo", serie::graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "branch_005");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn merge_001() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10");
    git.commit("011", "2024-02-01");

    git.checkout("master");
    git.checkout_b("20");
    git.commit("021", "2024-02-02");

    git.checkout("master");
    git.checkout_b("30");
    git.commit("031", "2024-02-03");

    git.checkout("10");
    git.commit("012", "2024-02-04");

    git.checkout("20");
    git.merge(&["10"], "2024-03-01");

    git.checkout("30");
    git.merge(&["10"], "2024-03-02");

    git.checkout("20");
    git.commit("022", "2024-03-03");

    git.checkout_b("40");
    git.commit("041", "2024-03-04");

    git.checkout("10");
    git.merge(&["20"], "2024-03-05");

    git.checkout("30");
    git.commit("032", "2024-03-06");

    git.checkout("10");
    git.merge(&["30"], "2024-03-07");

    git.checkout("40");
    git.merge(&["10"], "2024-03-08");

    git.checkout("master");
    git.merge(&["10"], "2024-03-09");

    git.log();

    let options = &[
        GenerateGraphOption::new("merge_001_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("merge_001_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "merge_001");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn merge_002() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10");
    git.commit("011", "2024-02-01");
    git.commit("012", "2024-02-02");

    git.checkout("master");
    git.checkout_b("20");
    git.commit("021", "2024-02-03");
    git.commit("022", "2024-02-04");

    git.checkout("master");
    git.checkout_b("30");
    git.commit("031", "2024-02-05");
    git.commit("032", "2024-02-06");

    git.checkout_b("40");
    git.commit("041", "2024-02-07");

    git.checkout("20");
    git.merge(&["10", "30"], "2024-03-01");

    git.checkout("master");
    git.merge(&["40"], "2024-03-02");

    git.log();

    let options = &[
        GenerateGraphOption::new("merge_002_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("merge_002_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "merge_002");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn merge_003() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10a");
    git.commit("011", "2024-02-01");

    git.checkout("master");
    git.checkout_b("20");
    git.commit("021", "2024-02-02");

    git.checkout("master");
    git.checkout_b("30");
    git.commit("031", "2024-02-03");

    git.checkout("10a");
    git.checkout_b("10b");
    git.checkout("10a");
    git.commit("012", "2024-02-04");

    git.checkout("20");
    git.merge(&["10a"], "2024-03-01");

    git.checkout("30");
    git.merge(&["10b"], "2024-03-02");

    git.checkout("master");
    git.merge(&["10a"], "2024-04-01");

    git.log();

    let options = &[
        GenerateGraphOption::new("merge_003_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("merge_003_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "merge_003");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn merge_004() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10a");
    git.commit("011", "2024-02-01");

    git.checkout("master");
    git.checkout_b("20");
    git.commit("021", "2024-02-02");

    git.checkout("master");
    git.checkout_b("30");
    git.commit("031", "2024-02-03");

    git.checkout("master");
    git.checkout_b("40");
    git.commit("041", "2024-02-04");

    git.checkout("10a");
    git.checkout_b("10c");

    git.checkout("10a");
    git.commit("012", "2024-02-05");

    git.checkout_b("10b");
    git.checkout("10a");
    git.commit("013", "2024-02-06");

    git.checkout("20");
    git.merge(&["10a"], "2024-03-01");

    git.checkout("30");
    git.merge(&["10b"], "2024-03-02");

    git.checkout("40");
    git.merge(&["10c"], "2024-03-03");

    git.checkout("master");
    git.merge(&["10a"], "2024-04-01");

    git.log();

    let options = &[
        GenerateGraphOption::new("merge_004_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("merge_004_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "merge_004");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn merge_005() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10");
    git.commit("011", "2024-02-01");

    git.checkout_b("20");
    git.commit("021", "2024-02-02");

    git.checkout("10");
    git.commit("012", "2024-02-03");

    git.checkout("master");
    git.merge(&["10"], "2024-03-01");

    git.checkout_b("30");
    git.commit("031", "2024-04-01");
    git.commit("032", "2024-04-02");

    git.checkout("master");
    git.commit("002", "2024-04-03");

    git.checkout_b("40");
    git.commit("041", "2024-05-01");

    git.checkout("master");
    git.merge(&["40"], "2024-05-02");

    git.checkout_b("50");
    git.checkout_b("60");

    git.checkout("50");
    git.commit("051", "2024-06-01");

    git.checkout("60");
    git.commit("061", "2024-06-02");

    git.checkout("master");
    git.merge(&["60"], "2024-06-03");

    git.checkout("master");
    git.merge(&["30"], "2024-06-04");

    git.checkout("master");
    git.merge(&["20"], "2024-06-05");

    git.log();

    let options = &[
        GenerateGraphOption::new("merge_005_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("merge_005_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "merge_005");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn stash_001() -> TestResult {
    // Test case for multiple stashes, the most recent commit is normal commit
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.stash("2024-01-03");

    git.commit("003", "2024-01-04");

    git.stash("2024-01-05");

    git.commit("004", "2024-01-06");

    git.checkout_b("10");
    git.checkout("master");

    git.commit("005", "2024-01-07");
    git.commit("006", "2024-01-08");

    git.checkout("10");
    git.stash("2024-01-09");

    git.checkout("master");
    git.commit("007", "2024-01-10");

    let options = &[
        GenerateGraphOption::new("stash_001_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("stash_001_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "stash_001");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn stash_002() -> TestResult {
    // Test case for multiple stashes, the most recent commit is stash
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.stash("2024-01-03");

    git.commit("003", "2024-01-04");

    git.stash("2024-01-05");

    git.commit("004", "2024-01-06");

    git.checkout_b("10");
    git.checkout("master");

    git.commit("005", "2024-01-07");
    git.commit("006", "2024-01-08");

    git.checkout("10");
    git.stash("2024-01-09");

    let options = &[
        GenerateGraphOption::new("stash_002_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("stash_002_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "stash_002");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn stash_003() -> TestResult {
    // Test case for unreachable stash
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.checkout_b("10");
    git.commit("011", "2024-02-01");

    git.stash("2024-02-02");

    git.checkout("master");
    git.commit("003", "2024-03-01");

    git.branch_d("10");

    let options = &[
        GenerateGraphOption::new("stash_003_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("stash_003_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "stash_003");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn stash_004() -> TestResult {
    // Test case for multiple stashes for the same commit
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.stash("2024-02-01");
    git.stash("2024-02-02");
    git.stash("2024-02-03");

    git.commit("003", "2024-03-01");

    let options = &[
        GenerateGraphOption::new("stash_004_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("stash_004_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "stash_004");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn orphan_001() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.checkout_orphan("o1");
    git.commit("011", "2024-01-03");

    git.checkout("master");
    git.commit("003", "2024-01-04");

    git.checkout("o1");
    git.commit("012", "2024-01-05");

    git.checkout("master");
    git.commit("004", "2024-01-06");

    git.checkout_orphan("o2");
    git.commit("021", "2024-01-07");
    git.commit("022", "2024-01-08");

    git.log();

    let options = &[
        GenerateGraphOption::new("orphan_001_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("orphan_001_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "orphan_001");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn orphan_002() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.checkout_b("010");
    git.commit("011", "2024-01-03");

    git.checkout("master");
    git.merge(&["010"], "2024-01-04");

    git.commit("003", "2024-02-01");

    git.checkout_orphan("o1");
    git.commit("021", "2024-02-02");
    git.commit("022", "2024-02-03");

    git.checkout("master");
    git.commit("004", "2024-02-04");
    git.commit("005", "2024-02-05");

    git.log();

    let options = &[
        GenerateGraphOption::new("orphan_002_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("orphan_002_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "orphan_002");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn complex_001() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10");
    git.checkout_b("20");

    git.checkout("master");
    git.commit("002", "2024-01-02");

    git.checkout("20");
    git.commit("021", "2024-02-01");

    git.checkout("10");
    git.commit("011", "2024-02-02");
    git.commit("012", "2024-02-03");

    git.checkout("master");
    git.checkout_b("30");
    git.commit("031", "2024-02-04");

    git.checkout("10");
    git.commit("013", "2024-03-01");

    git.checkout_b("40");

    git.checkout("20");
    git.merge(&["10"], "2024-03-02");
    git.commit("022", "2024-03-03");

    git.checkout("master");
    git.merge(&["30"], "2024-03-03");
    git.commit("003", "2024-03-04");

    git.checkout("40");
    git.merge(&["master"], "2024-04-01");
    git.commit("041", "2024-04-02");

    git.checkout("master");
    git.merge(&["40"], "2024-04-03");

    git.checkout("20");
    git.checkout_b("50");

    git.checkout("20");
    git.commit("023", "2024-05-01");
    git.commit("024", "2024-05-02");

    git.checkout("50");
    git.merge(&["20"], "2024-05-03");
    git.commit("051", "2024-05-04");

    git.checkout("20");
    git.merge(&["50"], "2024-05-05");

    git.checkout("30");
    git.commit("032", "2024-06-01");

    git.checkout("20");
    git.commit("025", "2024-06-02");

    git.log();

    let options = &[
        GenerateGraphOption::new("complex_001_chrono", graph::SortCommit::Chronological),
        GenerateGraphOption::new("complex_001_topo", graph::SortCommit::Topological),
    ];

    copy_git_dir(repo_path, "complex_001");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

struct GitRepository<'a> {
    path: &'a Path,
}

impl GitRepository<'_> {
    fn new(path: &Path) -> GitRepository {
        GitRepository { path }
    }

    fn init(&self) {
        self.run(&["init", "-b", "master"], "");
    }

    fn commit(&self, message: &str, date: &str) {
        let datetime_str = parse_date(date).to_rfc3339();
        self.run(&["commit", "--allow-empty", "-m", message], &datetime_str);
    }

    fn checkout(&self, branch_name: &str) {
        self.run(&["checkout", branch_name], "");
    }

    fn checkout_b(&self, branch_name: &str) {
        self.run(&["checkout", "-b", branch_name], "");
    }

    fn checkout_orphan(&self, branch_name: &str) {
        self.run(&["checkout", "--orphan", branch_name], "");
    }

    fn merge(&self, branch_names: &[&str], date: &str) {
        let datetime_str = parse_date(date).to_rfc3339();
        let mut args = vec!["merge", "--no-ff", "--no-log"];
        args.extend_from_slice(branch_names);
        self.run(&args, &datetime_str);
    }

    fn branch_d(&self, branch_name: &str) {
        self.run(&["branch", "-D", branch_name], "");
    }

    fn stash(&self, date: &str) {
        let dummy_file_path = self.path.join("stash.txt");
        std::fs::File::create(dummy_file_path).unwrap();

        let datetime_str = parse_date(date).to_rfc3339();
        self.run(&["stash", "--include-untracked"], &datetime_str);
    }

    fn log(&self) {
        let output = self.run(&["log", "--pretty=format:%h %s", "--graph", "--all"], "");
        println!("{}", String::from_utf8(output.stdout).unwrap())
    }

    fn run(&self, args: &[&str], datetime_str: &str) -> std::process::Output {
        Command::new("git")
            .args(args)
            .current_dir(self.path)
            .env("GIT_AUTHOR_NAME", "Author Name")
            .env("GIT_AUTHOR_EMAIL", "author@example.com")
            .env("GIT_AUTHOR_DATE", datetime_str)
            .env("GIT_COMMITTER_NAME", "Committer Name")
            .env("GIT_COMMITTER_EMAIL", "committer@example.com")
            .env("GIT_COMMITTER_DATE", datetime_str)
            .env("GIT_CONFIG_NOSYSTEM", "true")
            .env("HOME", "/dev/null")
            .output()
            .unwrap_or_else(|_| panic!("failed to execute git {}", args.join(" ")))
    }
}

fn parse_date(date: &str) -> DateTime<Utc> {
    let dt = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .unwrap()
        .and_hms_opt(1, 2, 3)
        .unwrap();
    Utc.from_utc_datetime(&dt)
}

struct GenerateGraphOption<'a> {
    output_name: &'a str,
    sort: graph::SortCommit,
}

impl GenerateGraphOption<'_> {
    fn new(output_name: &str, sort: graph::SortCommit) -> GenerateGraphOption {
        GenerateGraphOption { output_name, sort }
    }
}

fn generate_and_output_graph_images(repo_path: &Path, options: &[GenerateGraphOption]) {
    for option in options {
        generate_and_output_graph_image(repo_path, option);
    }
}

fn generate_and_output_graph_image<P: AsRef<Path>>(path: P, option: &GenerateGraphOption) {
    // Build graphs in the same way as application
    let graph_color_config = GraphColorConfig::default();
    let color_set = color::ColorSet::new(&graph_color_config);
    let repository = git::Repository::load(path.as_ref(), option.sort);
    let graph = graph::calc_graph(&repository);
    let image_params = graph::ImageParams::new(&color_set);
    let drawing_pixels = graph::DrawingPixels::new(&image_params);
    let graph_image = graph::build_graph_image(&graph, &image_params, &drawing_pixels);

    // Create concatenated image
    let (width, height) = (50, 50);
    let image_width = ((width * (graph.max_pos_x as usize + 1)) + (width * 7)) as u32;
    let image_height = (height * graph.commits.len()) as u32;
    let mut img_buf: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
        image::ImageBuffer::new(image_width, image_height);

    let text_renderer = text_to_png::TextRenderer::default();
    let text_x = (width * (graph.max_pos_x as usize + 1)) as u32;

    for (i, edges) in graph.edges.iter().enumerate() {
        let y = (height * i) as u32;

        // write graph
        let graph_row_image = &graph_image.images[edges];
        let image = image::load_from_memory(&graph_row_image.bytes).unwrap();
        img_buf.copy_from(&image, 0, y).unwrap();

        // write hash and date
        let commit = &graph.commits[i];
        let text = format!(
            "{} / {}",
            &commit.commit_hash.as_short_hash(),
            commit.committer_date.naive_utc().format("%Y-%m-%d")
        );
        let text_png = text_renderer
            .render_text_to_png_data(text, height / 4, 0x888888)
            .unwrap();
        let text_image = image::load_from_memory(&text_png.data).unwrap();
        img_buf
            .copy_from(&text_image, text_x, y + (height as u32 / 4))
            .unwrap();

        // write subject
        let text = &commit.subject;
        let text_png = text_renderer
            .render_text_to_png_data(text, height / 4, 0x888888)
            .unwrap();
        let text_image = image::load_from_memory(&text_png.data).unwrap();
        img_buf
            .copy_from(&text_image, text_x, y + ((height as u32 / 4) * 2))
            .unwrap();
    }

    // Save
    create_output_dirs(OUTPUT_DIR);
    let file_name = format!("{}/{}.png", OUTPUT_DIR, option.output_name);
    image::save_buffer(
        file_name,
        &img_buf,
        image_width,
        image_height,
        image::ColorType::Rgba8,
    )
    .unwrap();
}

fn create_output_dirs(path: &str) {
    let path = Path::new(path);
    std::fs::create_dir_all(path).unwrap();
}

fn copy_git_dir(path: &Path, name: &str) {
    let dst_path = format!("{}/{}", OUTPUT_DIR, name);
    // dircpy overwrite doesn't seem to work as expected, so delete explicitly
    if Path::new(&dst_path).is_dir() {
        std::fs::remove_dir_all(&dst_path).unwrap();
    }
    dircpy::CopyBuilder::new(path, dst_path).run().unwrap();
}

fn assert_graph_images(options: &[GenerateGraphOption]) {
    let errors: Vec<_> = options
        .iter()
        .map(compare_graph_image)
        .filter_map(Result::err)
        .collect();
    if !errors.is_empty() {
        panic!("{}", errors.join("\n"));
    }
}

fn compare_graph_image(option: &GenerateGraphOption) -> Result<(), String> {
    let expected_file = format!("{}/{}.png", SNAPSHOT_DIR, option.output_name);
    let expected_img = image::open(expected_file).unwrap();

    let actual_file = format!("{}/{}.png", OUTPUT_DIR, option.output_name);
    let actual_img = image::open(actual_file).unwrap();

    if actual_img.dimensions() != expected_img.dimensions() {
        return Err(format!(
            "Image dimensions are different. expected: {:?}, actual: {:?}",
            expected_img.dimensions(),
            actual_img.dimensions()
        ));
    }

    let (image_width, image_height) = actual_img.dimensions();
    let mut img_buf = image::ImageBuffer::new(image_width, image_height);
    let mut diff = false;

    for y in 0..image_height {
        for x in 0..image_width {
            let actual_pixel = actual_img.get_pixel(x, y);
            let expected_pixel = expected_img.get_pixel(x, y);

            if actual_pixel != expected_pixel {
                img_buf.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
                diff = true;
            } else {
                img_buf.put_pixel(x, y, actual_pixel);
            }
        }
    }

    if diff {
        let diff_file = format!("{}/{}_diff.png", OUTPUT_DIR, option.output_name);
        image::save_buffer(
            diff_file.clone(),
            &img_buf,
            image_width,
            image_height,
            image::ColorType::Rgba8,
        )
        .unwrap();

        return Err(format!("Images are different. diff: {}", diff_file));
    }

    Ok(())
}
