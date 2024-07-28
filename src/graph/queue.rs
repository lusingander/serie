use std::collections::BinaryHeap;

use crate::git::Commit;

struct WrappedCommit<'a>(&'a Commit);

impl PartialEq for WrappedCommit<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.committer_date_sort_key() == other.0.committer_date_sort_key()
    }
}

impl Eq for WrappedCommit<'_> {}

impl PartialOrd for WrappedCommit<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WrappedCommit<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .0
            .committer_date_sort_key()
            .cmp(&self.0.committer_date_sort_key())
    }
}

pub struct PriorityQueue<'a> {
    heap: BinaryHeap<WrappedCommit<'a>>,
}

impl<'a> PriorityQueue<'a> {
    pub fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
        }
    }

    pub fn enqueue(&mut self, c: &'a Commit) {
        self.heap.push(WrappedCommit(c));
    }

    pub fn dequeue(&mut self) -> Option<&'a Commit> {
        self.heap.pop().map(|WrappedCommit(c)| c)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local, NaiveDate, TimeZone};

    use crate::git::CommitHash;

    use super::*;

    #[test]
    fn test_priority_queue() {
        let mut queue = PriorityQueue::new();
        let c1 = commit("1", &[], "2024-01-04");
        let c2 = commit("2", &[], "2024-01-01");
        let c3 = commit("3", &[], "2024-01-02");
        let c4 = commit("4", &[], "2024-01-05");
        let c5 = commit("5", &[], "2024-01-03");

        queue.enqueue(&c1);
        queue.enqueue(&c2);

        assert_eq!(queue.dequeue().unwrap().commit_hash.as_short_hash(), "2");

        queue.enqueue(&c3);
        queue.enqueue(&c4);
        queue.enqueue(&c5);

        assert_eq!(queue.dequeue().unwrap().commit_hash.as_short_hash(), "3");
        assert_eq!(queue.dequeue().unwrap().commit_hash.as_short_hash(), "5");
        assert_eq!(queue.dequeue().unwrap().commit_hash.as_short_hash(), "1");
        assert_eq!(queue.dequeue().unwrap().commit_hash.as_short_hash(), "4");
    }

    fn commit(hash: &str, parent_hashes: &[&str], date: &str) -> Commit {
        Commit {
            commit_hash: hash.into(),
            committer_date: parse_date(date),
            parent_commit_hashes: parent_hashes.iter().map(|s| CommitHash::from(*s)).collect(),
            ..Default::default()
        }
    }

    fn parse_date(date: &str) -> DateTime<Local> {
        let local = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(1, 2, 3)
            .unwrap();
        Local.from_local_datetime(&local).unwrap()
    }
}
