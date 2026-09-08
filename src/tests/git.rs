use std::{path::Path, process::Command};

use chrono::{DateTime, NaiveDate, TimeZone, Utc};

pub(crate) struct GitRepository<'a> {
    path: &'a Path,
    author_name: String,
    author_email: String,
    committer_name: String,
    committer_email: String,
}

impl<'a> GitRepository<'a> {
    pub(crate) fn new(path: &'a Path) -> GitRepository<'a> {
        GitRepository {
            path,
            author_name: "Author Name".to_string(),
            author_email: "author@example.com".to_string(),
            committer_name: "Committer Name".to_string(),
            committer_email: "committer@example.com".to_string(),
        }
    }

    pub(crate) fn with_identities(
        mut self,
        author_name: &str,
        author_email: &str,
        committer_name: &str,
        committer_email: &str,
    ) -> Self {
        self.author_name = author_name.to_string();
        self.author_email = author_email.to_string();
        self.committer_name = committer_name.to_string();
        self.committer_email = committer_email.to_string();
        self
    }

    pub(crate) fn init(&self) {
        self.run(&["init", "-b", "master"], "");
    }

    pub(crate) fn commit(&self, message: &str, date: &str) {
        let datetime_str = parse_date(date).to_rfc3339();
        self.run(&["commit", "--allow-empty", "-m", message], &datetime_str);
    }

    pub(crate) fn checkout(&self, branch_name: &str) {
        self.run(&["checkout", branch_name], "");
    }

    pub(crate) fn checkout_b(&self, branch_name: &str) {
        self.run(&["checkout", "-b", branch_name], "");
    }

    pub(crate) fn checkout_orphan(&self, branch_name: &str) {
        self.run(&["checkout", "--orphan", branch_name], "");
    }

    pub(crate) fn merge(&self, branch_names: &[&str], date: &str) {
        let datetime_str = parse_date(date).to_rfc3339();
        let mut args = vec!["merge", "--no-ff", "--no-log"];
        args.extend_from_slice(branch_names);
        self.run(&args, &datetime_str);
    }

    pub(crate) fn branch_d(&self, branch_name: &str) {
        self.run(&["branch", "-D", branch_name], "");
    }

    pub(crate) fn stash(&self, date: &str) {
        let dummy_file_path = self.path.join("stash.txt");
        std::fs::File::create(dummy_file_path).unwrap();

        let datetime_str = parse_date(date).to_rfc3339();
        self.run(&["stash", "--include-untracked"], &datetime_str);
    }

    pub(crate) fn rev_parse_head(&self) -> String {
        let output = self.run(&["rev-parse", "HEAD"], "");
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    pub(crate) fn log(&self) {
        let output = self.run(&["log", "--pretty=format:%h %s", "--graph", "--all"], "");
        println!("{}", String::from_utf8(output.stdout).unwrap())
    }

    fn run(&self, args: &[&str], datetime_str: &str) -> std::process::Output {
        let output = Command::new("git")
            .args(args)
            .current_dir(self.path)
            .env("GIT_AUTHOR_NAME", &self.author_name)
            .env("GIT_AUTHOR_EMAIL", &self.author_email)
            .env("GIT_AUTHOR_DATE", datetime_str)
            .env("GIT_COMMITTER_NAME", &self.committer_name)
            .env("GIT_COMMITTER_EMAIL", &self.committer_email)
            .env("GIT_COMMITTER_DATE", datetime_str)
            .env("GIT_CONFIG_NOSYSTEM", "true")
            .env("HOME", "/dev/null")
            .output()
            .unwrap_or_else(|error| panic!("failed to execute git {}: {error}", args.join(" ")));

        assert!(
            output.status.success(),
            "git {} failed with {}\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        output
    }
}

fn parse_date(date: &str) -> DateTime<Utc> {
    let dt = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .unwrap()
        .and_hms_opt(1, 2, 3)
        .unwrap();
    Utc.from_utc_datetime(&dt)
}
