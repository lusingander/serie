use std::path::Path;

use chrono::{Days, TimeZone, Utc};
use image::{GenericImage, GenericImageView};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    color, config, git,
    graph::{self, Edge, GraphRowImage},
    test_git::GitRepository,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const OUTPUT_DIR: &str = "./out/graph";
const SNAPSHOT_DIR: &str = "./tests/graph";
// Set this environment variable when Git repository output is needed for graph test debugging.
const DUMP_GRAPH_TEST_REPOS_ENV: &str = "SERIE_TEST_DUMP_GRAPH_REPOS";

#[test]
fn straight_001() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    let mut base_date = Utc.with_ymd_and_hms(2024, 1, 1, 1, 2, 3).unwrap();
    for i in 1..=100 {
        let msg = &format!("{i:03}");
        let date = &base_date.format("%Y-%m-%d").to_string();
        git.commit(msg, date);
        base_date = base_date.checked_add_days(Days::new(1)).unwrap();
    }

    git.log();

    let options = &[GenerateGraphOption::new(
        "straight_001",
        git::SortCommit::Chronological,
        graph::GraphStyle::Rounded,
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
        GenerateGraphOption::new(
            "branch_001_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_001_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_001_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
        GenerateGraphOption::new(
            "branch_001_curved",
            git::SortCommit::Chronological,
            graph::GraphStyle::Curved,
        ),
        GenerateGraphOption::new(
            "branch_001_max_count",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        )
        .with_max_count(10),
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
        GenerateGraphOption::new(
            "branch_002_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_002_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_002_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
        GenerateGraphOption::new(
            "branch_002_max_count",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        )
        .with_max_count(5),
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
        GenerateGraphOption::new(
            "branch_003_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_003_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_003_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
        GenerateGraphOption::new(
            "branch_003_curved",
            git::SortCommit::Chronological,
            graph::GraphStyle::Curved,
        ),
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
        GenerateGraphOption::new(
            "branch_004_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_004_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_004_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "branch_005_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_005_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_005_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
    ];

    copy_git_dir(repo_path, "branch_005");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn branch_006() -> TestResult {
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
    git.checkout_b("30");

    git.checkout("master");
    git.merge(&["10"], "2024-03-01");

    git.checkout("20");
    git.commit("021", "2024-03-02");

    git.checkout("master");
    git.merge(&["20"], "2024-03-03");

    git.checkout("30");
    git.commit("031", "2024-03-04");

    git.checkout("master");
    git.merge(&["30"], "2024-03-05");

    git.log();

    let options = &[
        GenerateGraphOption::new(
            "branch_006_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_006_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_006_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
    ];

    copy_git_dir(repo_path, "branch_006");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn branch_007() -> TestResult {
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

    git.log();

    let options = &[
        GenerateGraphOption::new(
            "branch_007_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_007_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_007_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
    ];

    copy_git_dir(repo_path, "branch_007");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn branch_008() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");

    git.checkout_b("10");

    git.checkout("master");
    git.commit("002", "2024-01-02");

    git.checkout("10");
    git.commit("011", "2024-04-01");

    git.checkout("master");
    git.commit("003", "2024-02-01");

    git.checkout_b("20");

    git.checkout("master");
    git.merge(&["10"], "2024-03-01");

    git.checkout("20");
    git.commit("021", "2024-03-01");

    git.checkout("master");
    git.merge(&["20"], "2024-03-02");

    git.log();

    let options = &[
        GenerateGraphOption::new(
            "branch_008_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_008_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "branch_008_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
    ];

    copy_git_dir(repo_path, "branch_008");

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
        GenerateGraphOption::new(
            "merge_001_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_001_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_001_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "merge_002_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_002_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_002_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "merge_003_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_003_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_003_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "merge_004_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_004_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_004_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
        GenerateGraphOption::new(
            "merge_004_curved",
            git::SortCommit::Chronological,
            graph::GraphStyle::Curved,
        ),
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
        GenerateGraphOption::new(
            "merge_005_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_005_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_005_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
    ];

    copy_git_dir(repo_path, "merge_005");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn merge_006() -> TestResult {
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

    git.merge(&["10"], "2024-03-02");

    git.checkout("10");
    git.commit("012", "2024-03-01");

    git.log();

    let options = &[
        GenerateGraphOption::new(
            "merge_006_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_006_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "merge_006_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
    ];

    copy_git_dir(repo_path, "merge_006");

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
        GenerateGraphOption::new(
            "stash_001_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "stash_001_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "stash_001_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "stash_002_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "stash_002_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "stash_002_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "stash_003_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "stash_003_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "stash_003_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "stash_004_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "stash_004_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "stash_004_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "orphan_001_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "orphan_001_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "orphan_001_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
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
        GenerateGraphOption::new(
            "orphan_002_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "orphan_002_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "orphan_002_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
    ];

    copy_git_dir(repo_path, "orphan_002");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn head_001() -> TestResult {
    // Test case for detached HEAD
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.checkout_b("10");
    git.commit("011", "2024-01-03");
    git.commit("012", "2024-01-04");

    let hash = git.rev_parse_head();

    git.checkout("master");
    git.commit("003", "2024-01-05");
    git.commit("004", "2024-01-06");

    git.checkout(hash.as_str());
    git.branch_d("10");

    git.log();

    let options = &[
        GenerateGraphOption::new(
            "head_001_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "head_001_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "head_001_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
    ];

    copy_git_dir(repo_path, "head_001");

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
        GenerateGraphOption::new(
            "complex_001_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "complex_001_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        ),
        GenerateGraphOption::new(
            "complex_001_angular",
            git::SortCommit::Chronological,
            graph::GraphStyle::Angular,
        ),
        GenerateGraphOption::new(
            "complex_001_curved",
            git::SortCommit::Chronological,
            graph::GraphStyle::Curved,
        ),
    ];

    copy_git_dir(repo_path, "complex_001");

    generate_and_output_graph_images(repo_path, options);
    assert_graph_images(options);

    Ok(())
}

#[test]
fn primary_branch_001() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();

    let git = &GitRepository::new(repo_path);

    git.init();

    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.checkout_b("feature1");
    git.commit("feat1", "2024-01-03");

    git.checkout("master");
    git.commit("003", "2024-01-04");

    git.checkout_b("feature2");
    git.commit("feat2", "2024-01-05");

    git.checkout("master");
    git.merge(&["feature1"], "2024-01-06");

    git.checkout("feature2");
    git.commit("feat2-2", "2024-01-07");

    git.checkout("master");
    git.checkout_b("feature3");
    git.commit("feat3", "2024-01-08");

    git.log();

    let options = &[
        GenerateGraphOption::new(
            "primary_branch_001_chrono",
            git::SortCommit::Chronological,
            graph::GraphStyle::Rounded,
        )
        .with_primary_branch("master"),
        GenerateGraphOption::new(
            "primary_branch_001_topo",
            git::SortCommit::Topological,
            graph::GraphStyle::Rounded,
        )
        .with_primary_branch("master"),
    ];

    copy_git_dir(repo_path, "primary_branch_001");

    generate_and_output_graph_images(repo_path, options);

    let normal_repository =
        git::Repository::load(repo_path, git::SortCommit::Chronological, None, true)?;
    let normal_graph = graph::calc_graph(&normal_repository, None);
    let normal_feature_tip = normal_graph
        .commits
        .iter()
        .find(|c| c.subject == "feat3")
        .unwrap();
    assert_eq!(
        normal_graph.commit_pos_map[&normal_feature_tip.commit_hash].0,
        0
    );

    for sort in [git::SortCommit::Chronological, git::SortCommit::Topological] {
        let repository = git::Repository::load(repo_path, sort, None, true)?;
        let graph = graph::calc_graph(&repository, Some("master"));

        assert_eq!(graph.warning, None);

        for subject in ["001", "002", "003"] {
            let commit = graph.commits.iter().find(|c| c.subject == subject).unwrap();
            assert_eq!(
                graph.commit_pos_map[&commit.commit_hash].0, 0,
                "primary branch commit {subject} was not pinned to column 0"
            );
        }

        let merge_commit = graph
            .commits
            .iter()
            .find(|c| c.parent_commit_hashes.len() == 2)
            .unwrap();
        assert_eq!(
            graph.commit_pos_map[&merge_commit.commit_hash].0, 0,
            "primary branch merge commit was not pinned to column 0"
        );

        for subject in ["feat1", "feat2", "feat2-2", "feat3"] {
            let commit = graph.commits.iter().find(|c| c.subject == subject).unwrap();
            assert!(
                graph.commit_pos_map[&commit.commit_hash].0 >= 1,
                "feature commit {subject} was assigned to column 0"
            );
        }
    }

    assert_graph_images(options);

    Ok(())
}

#[test]
fn primary_branch_merge_back() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();
    let git = &GitRepository::new(repo_path);

    git.init();
    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.checkout_b("feature");
    git.commit("feat1", "2024-01-03");
    git.commit("feat2", "2024-01-04");

    git.checkout("master");
    git.commit("003", "2024-01-05");
    git.merge(&["feature"], "2024-01-06");

    // Add another commit to master after merge
    git.commit("004", "2024-01-07");

    // Also a newer feature branch commit
    git.checkout("feature");
    git.commit("feat3", "2024-01-08");
    git.checkout("master");

    let repository = git::Repository::load(repo_path, git::SortCommit::Chronological, None, true)?;
    let graph = graph::calc_graph(&repository, Some("master"));

    let m4 = graph.commits.iter().find(|c| c.subject == "004").unwrap();
    let merge_commit = graph
        .commits
        .iter()
        .find(|c| c.parent_commit_hashes.len() == 2)
        .unwrap();
    let m3 = graph.commits.iter().find(|c| c.subject == "003").unwrap();
    let m2 = graph.commits.iter().find(|c| c.subject == "002").unwrap();
    let m1 = graph.commits.iter().find(|c| c.subject == "001").unwrap();
    let feat3 = graph.commits.iter().find(|c| c.subject == "feat3").unwrap();
    let feat2 = graph.commits.iter().find(|c| c.subject == "feat2").unwrap();
    let feat1 = graph.commits.iter().find(|c| c.subject == "feat1").unwrap();

    // Primary branch spine commits must all be on column 0
    assert_eq!(graph.commit_pos_map[&m4.commit_hash].0, 0);
    assert_eq!(graph.commit_pos_map[&merge_commit.commit_hash].0, 0);
    assert_eq!(graph.commit_pos_map[&m3.commit_hash].0, 0);
    assert_eq!(graph.commit_pos_map[&m2.commit_hash].0, 0);
    assert_eq!(graph.commit_pos_map[&m1.commit_hash].0, 0);

    // Feature branch commits must be placed in columns >= 1
    assert!(graph.commit_pos_map[&feat3.commit_hash].0 >= 1);
    assert!(graph.commit_pos_map[&feat2.commit_hash].0 >= 1);
    assert!(graph.commit_pos_map[&feat1.commit_hash].0 >= 1);

    Ok(())
}

#[test]
fn primary_branch_fallback_unresolved_or_none() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();
    let git = &GitRepository::new(repo_path);

    git.init();
    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");
    git.checkout_b("feature");
    git.commit("feat1", "2024-01-04");
    git.checkout("master");
    git.commit("003", "2024-01-03");

    let repository = git::Repository::load(repo_path, git::SortCommit::Chronological, None, true)?;

    let graph_none = graph::calc_graph(&repository, None);
    let graph_unknown = graph::calc_graph(&repository, Some("nonexistent_branch"));

    // Unknown branch must fall back gracefully to the normal layout
    assert_eq!(graph_none.commit_pos_map, graph_unknown.commit_pos_map);
    assert_eq!(graph_none.warning, None);
    assert_eq!(
        graph_unknown.warning.as_deref(),
        Some("Primary branch 'nonexistent_branch' could not be resolved")
    );

    Ok(())
}

#[test]
fn primary_branch_max_count() -> TestResult {
    let dir = tempfile::tempdir()?;
    let repo_path = dir.path();
    let git = &GitRepository::new(repo_path);

    git.init();
    git.commit("001", "2024-01-01");
    git.commit("002", "2024-01-02");

    git.checkout_b("feature");
    git.commit("feat1", "2024-01-03");
    git.commit("feat2", "2024-01-04");
    git.commit("feat3", "2024-01-05");

    git.checkout("master");

    // Case 1: max_count = 2 (only feat3 and feat2 loaded, primary branch master tip is excluded)
    // Must return warning and fall back to the normal graph layout.
    let repo_truncated =
        git::Repository::load(repo_path, git::SortCommit::Chronological, Some(2), true)?;
    let graph_truncated = graph::calc_graph(&repo_truncated, Some("master"));
    let graph_normal = graph::calc_graph(&repo_truncated, None);
    assert_eq!(graph_truncated.commit_pos_map, graph_normal.commit_pos_map);
    assert_eq!(
        graph_truncated.warning.as_deref(),
        Some("Primary branch 'master' tip is not included in the loaded commits")
    );

    // Case 2: max_count = 5 (all commits loaded, primary branch master tip is visible)
    // Feature commits must be on pos_x = 1, and master commits on pos_x = 0.
    let repo_full =
        git::Repository::load(repo_path, git::SortCommit::Chronological, Some(5), true)?;
    let graph_full = graph::calc_graph(&repo_full, Some("master"));
    let feat3_full = graph_full
        .commits
        .iter()
        .find(|c| c.subject == "feat3")
        .unwrap();
    let m2_full = graph_full
        .commits
        .iter()
        .find(|c| c.subject == "002")
        .unwrap();
    assert_eq!(graph_full.commit_pos_map[&feat3_full.commit_hash].0, 1);
    assert_eq!(graph_full.commit_pos_map[&m2_full.commit_hash].0, 0);

    // Case 3: max_count = 4 (feat3, feat2, feat1, 002 loaded; master tip 002 is included, older 001 truncated)
    // Pinning the visible part of the spine to column 0.
    let repo_part =
        git::Repository::load(repo_path, git::SortCommit::Chronological, Some(4), true)?;
    let graph_part = graph::calc_graph(&repo_part, Some("master"));
    let feat3_part = graph_part
        .commits
        .iter()
        .find(|c| c.subject == "feat3")
        .unwrap();
    let m2_part = graph_part
        .commits
        .iter()
        .find(|c| c.subject == "002")
        .unwrap();
    assert_eq!(graph_part.commit_pos_map[&feat3_part.commit_hash].0, 1);
    assert_eq!(graph_part.commit_pos_map[&m2_part.commit_hash].0, 0);

    Ok(())
}

struct GenerateGraphOption {
    output_name: &'static str,
    sort: git::SortCommit,
    style: graph::GraphStyle,
    max_count: Option<usize>,
    primary_branch: Option<String>,
}

impl GenerateGraphOption {
    fn new(
        output_name: &'static str,
        sort: git::SortCommit,
        style: graph::GraphStyle,
    ) -> GenerateGraphOption {
        GenerateGraphOption {
            output_name,
            sort,
            style,
            max_count: None,
            primary_branch: None,
        }
    }

    fn with_max_count(mut self, max_count: usize) -> GenerateGraphOption {
        self.max_count = Some(max_count);
        self
    }

    fn with_primary_branch(mut self, primary_branch: impl Into<String>) -> GenerateGraphOption {
        self.primary_branch = Some(primary_branch.into());
        self
    }
}

fn generate_and_output_graph_images(repo_path: &Path, options: &[GenerateGraphOption]) {
    for option in options {
        generate_and_output_graph_image(repo_path, option);
    }
}

fn generate_and_output_graph_image<P: AsRef<Path>>(path: P, option: &GenerateGraphOption) {
    // Build graphs in the same way as application
    let max_count = option.max_count;
    let graph_color_config = config::GraphColorConfig::default();
    let graph_color_set = color::GraphColorSet::new(&graph_color_config);
    let cell_width_type = graph::CellWidthType::Double;
    let repository = git::Repository::load(path.as_ref(), option.sort, max_count, true).unwrap();
    let graph = graph::calc_graph(&repository, option.primary_branch.as_deref());
    let image_params = graph::ImageParams::new(&graph_color_set, cell_width_type);
    let drawing_pixels = graph::DrawingPixels::new(&image_params);
    let graph_image = build_graph_image(&graph, &image_params, &drawing_pixels, option.style);

    // Create concatenated image
    let (width, height) = (50, 50);
    let image_width = ((width * (graph.max_pos_x + 1)) + (width * 7)) as u32;
    let image_height = (height * graph.commits.len()) as u32;
    let mut img_buf: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
        image::ImageBuffer::new(image_width, image_height);

    let text_renderer = text_to_png::TextRenderer::default();
    let text_x = (width * (graph.max_pos_x + 1)) as u32;

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
            commit.commit_hash.as_short_hash(),
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

#[derive(Debug, Default)]
pub struct GraphImage {
    pub images: FxHashMap<Vec<Edge>, GraphRowImage>,
}

fn build_graph_image(
    graph: &graph::Graph<'_>,
    image_params: &graph::ImageParams,
    drawing_pixels: &graph::DrawingPixels,
    graph_style: graph::GraphStyle,
) -> GraphImage {
    let graph_row_sources: FxHashSet<(usize, &Vec<graph::Edge>)> = graph
        .commits
        .iter()
        .map(|commit| {
            let (pos_x, pos_y) = graph.commit_pos_map[&commit.commit_hash];
            let edges = &graph.edges[pos_y];
            (pos_x, edges)
        })
        .collect();

    let cell_count = graph.max_pos_x + 1;

    let images = graph_row_sources
        .into_iter()
        .map(|(pos_x, edges)| {
            let graph_row_image = graph::calc_graph_row_image(
                pos_x,
                cell_count,
                edges,
                image_params,
                drawing_pixels,
                graph_style,
            );
            (edges.clone(), graph_row_image)
        })
        .collect();

    GraphImage { images }
}

fn create_output_dirs(path: &str) {
    let path = Path::new(path);
    std::fs::create_dir_all(path).unwrap();
}

fn copy_git_dir(path: &Path, name: &str) {
    if std::env::var_os(DUMP_GRAPH_TEST_REPOS_ENV).is_none() {
        return;
    }

    let dst_path = format!("{OUTPUT_DIR}/{name}");
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

        return Err(format!("Images are different. diff: {diff_file}"));
    }

    Ok(())
}
