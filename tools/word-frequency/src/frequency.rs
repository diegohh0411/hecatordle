use rustc_hash::FxHashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WordData {
    pub word: String,
    pub frequency_per_1k: f64,
    pub difficulty: String,
}

pub struct FrequencyCounter {
    word_counts: FxHashMap<String, usize>,
    total_counted: usize,
}

impl FrequencyCounter {
    pub fn new() -> Self {
        Self {
            word_counts: FxHashMap::with_capacity_and_hasher(15_000, Default::default()),
            total_counted: 0,
        }
    }

    /// Add a word by reference — only allocates if the key is new.
    pub fn add_str(&mut self, word: &str) {
        self.total_counted += 1;
        if let Some(count) = self.word_counts.get_mut(word) {
            *count += 1;
        } else {
            self.word_counts.insert(word.to_owned(), 1);
        }
    }

    /// Merge another counter into this one.
    pub fn merge(mut self, other: Self) -> Self {
        self.total_counted += other.total_counted;
        for (word, count) in other.word_counts {
            *self.word_counts.entry(word).or_insert(0) += count;
        }
        self
    }

    /// Consumes the counter and returns words sorted by frequency with difficulty labels.
    /// `easy_fraction` and `normal_fraction` are cumulative thresholds (e.g. 0.4 and 0.8).
    pub fn build(self, easy_fraction: f64, normal_fraction: f64) -> Vec<WordData> {
        let total_counted = self.total_counted;
        let mut results: Vec<WordData> = self.word_counts.into_iter().map(|(word, count)| {
            let frequency_per_1k = (count as f64 / total_counted as f64) * 1000.0;
            WordData { word, frequency_per_1k, difficulty: String::new() }
        }).collect();

        results.sort_by(|a, b| b.frequency_per_1k.partial_cmp(&a.frequency_per_1k)
            .unwrap_or(std::cmp::Ordering::Equal));

        let total_unique = results.len();
        let easy_threshold = (total_unique as f64 * easy_fraction) as usize;
        let normal_threshold = (total_unique as f64 * normal_fraction) as usize;

        for (i, data) in results.iter_mut().enumerate() {
            data.difficulty = if i < easy_threshold {
                "easy".to_string()
            } else if i < normal_threshold {
                "normal".to_string()
            } else {
                "hard".to_string()
            };
        }

        results
    }
}
