use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub struct Fuzzy {
    matcher: SkimMatcherV2,
}

impl Fuzzy {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn score(&self, query: &str, text: &str) -> Option<i64> {
        self.matcher.fuzzy_match(text, query)
    }

    pub fn filter<T, F>(&self, query: &str, items: &[T], f: F) -> Vec<(usize, i64)>
    where
        F: Fn(&T) -> &str,
    {
        let mut results: Vec<(usize, i64)> = items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let text = f(item);
                self.score(query, text).map(|score| (i, score))
            })
            .collect();
        results.sort_by_key(|k| std::cmp::Reverse(k.1));
        results
    }
}

impl Default for Fuzzy {
    fn default() -> Self {
        Self::new()
    }
}
