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
}

impl Default for Fuzzy {
    fn default() -> Self {
        Self::new()
    }
}
