use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchTarget {
    #[default]
    All,
    Subject,
    Author,
    Ref,
    Hash,
}

impl SearchTarget {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Subject,
            Self::Subject => Self::Author,
            Self::Author => Self::Ref,
            Self::Ref => Self::Hash,
            Self::Hash => Self::All,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Subject => "subject",
            Self::Author => "author",
            Self::Ref => "ref",
            Self::Hash => "hash",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SearchOptions {
    pub target: SearchTarget,
    pub ignore_case: bool,
    pub fuzzy: bool,
}

impl SearchOptions {
    pub fn status_string(&self) -> String {
        let case = if self.ignore_case {
            "ignore-case"
        } else {
            "case-sensitive"
        };
        let matcher = if self.fuzzy { "fuzzy" } else { "substring" };
        format!("[{}] [{case}] [{matcher}]", self.target.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_target_next_cycles_through_all_targets() {
        let mut target = SearchTarget::All;
        let expected = [
            SearchTarget::Subject,
            SearchTarget::Author,
            SearchTarget::Ref,
            SearchTarget::Hash,
            SearchTarget::All,
        ];

        for expected_target in expected {
            target = target.next();
            assert_eq!(target, expected_target);
        }
    }

    #[test]
    fn test_search_options_status_string() {
        let options = SearchOptions {
            target: SearchTarget::Author,
            ignore_case: true,
            fuzzy: true,
        };

        assert_eq!(options.status_string(), "[author] [ignore-case] [fuzzy]");
    }
}
