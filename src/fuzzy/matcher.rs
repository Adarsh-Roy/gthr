use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub score: i64,
    pub indices: Vec<usize>,
    pub item_index: usize,
}

impl MatchResult {
    pub fn new(score: i64, indices: Vec<usize>, item_index: usize) -> Self {
        Self {
            score,
            indices,
            item_index,
        }
    }
}

pub fn search_items<T, F>(
    items: &[T],
    query: &str,
    extract_text: F,
) -> Vec<MatchResult>
where
    F: Fn(&T) -> &str,
{
    if query.is_empty() {
        return (0..items.len())
            .map(|i| MatchResult::new(0, Vec::new(), i))
            .collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut results = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let text = extract_text(item);
        if let Some((score, indices)) = matcher.fuzzy_indices(text, query) {
            results.push(MatchResult::new(score, indices, index));
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_search() {
        let matcher = SkimMatcherV2::default();

        let result = matcher.fuzzy_indices("main.rs", "rs");
        assert!(result.is_some());

        let (score, indices) = result.unwrap();
        assert!(score > 0);
        assert_eq!(indices, vec![5, 6]);
    }

    #[test]
    fn test_search_items() {
        let items = vec!["main.rs", "lib.rs", "config.toml", "README.md"];
        let results = search_items(&items, "rs", |item| item);

        assert_eq!(results.len(), 2);
        assert!(results[0].score >= results[1].score);
    }
}
