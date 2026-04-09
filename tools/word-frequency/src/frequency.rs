use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub book: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEntry {
    pub word: String,
    pub frequency_per_1k: f64,
    pub difficulty: String,
    pub sources: Vec<SourceEntry>,
}

/// Result of processing a single book/file.
pub struct BookResult {
    pub book_name: String,
    pub word_counts: HashMap<String, usize>,
    pub total_words: usize,
}

impl BookResult {
    pub fn new(book_name: String, words: Vec<String>) -> Self {
        let total_words = words.len();
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }
        Self {
            book_name,
            word_counts,
            total_words,
        }
    }
}

/// Derive difficulty category from frequency per 1k words.
pub fn derive_difficulty(frequency_per_1k: f64) -> &'static str {
    if frequency_per_1k > 1.0 {
        "easy"
    } else if frequency_per_1k >= 0.1 {
        "normal"
    } else {
        "hard"
    }
}

/// Aggregate multiple book results into a list of WordEntry, sorted by frequency descending.
pub fn aggregate(books: &[BookResult]) -> Vec<WordEntry> {
    let total_corpus: usize = books.iter().map(|b| b.total_words).sum();

    // Collect per-word totals and per-source counts
    let mut global_counts: HashMap<String, usize> = HashMap::new();
    let mut per_source: HashMap<String, Vec<SourceEntry>> = HashMap::new();

    for book in books {
        for (word, &count) in &book.word_counts {
            *global_counts.entry(word.clone()).or_insert(0) += count;
            per_source
                .entry(word.clone())
                .or_default()
                .push(SourceEntry {
                    book: book.book_name.clone(),
                    count,
                });
        }
    }

    let mut entries: Vec<WordEntry> = global_counts
        .into_iter()
        .map(|(word, total_count)| {
            let frequency_per_1k = if total_corpus > 0 {
                (total_count as f64 / total_corpus as f64) * 1000.0
            } else {
                0.0
            };
            let difficulty = derive_difficulty(frequency_per_1k).to_string();
            let mut sources = per_source.remove(&word).unwrap_or_default();
            sources.sort_by(|a, b| b.count.cmp(&a.count));
            WordEntry {
                word,
                frequency_per_1k,
                difficulty,
                sources,
            }
        })
        .collect();

    entries.sort_by(|a, b| {
        b.frequency_per_1k
            .partial_cmp(&a.frequency_per_1k)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.word.cmp(&b.word))
    });

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_thresholds() {
        assert_eq!(derive_difficulty(1.1), "easy");
        assert_eq!(derive_difficulty(1.0), "normal");
        assert_eq!(derive_difficulty(0.5), "normal");
        assert_eq!(derive_difficulty(0.1), "normal");
        assert_eq!(derive_difficulty(0.09), "hard");
        assert_eq!(derive_difficulty(0.0), "hard");
    }

    #[test]
    fn test_aggregate_single_book() {
        let words = vec![
            "hello".to_string(),
            "world".to_string(),
            "hello".to_string(),
            "there".to_string(),
        ];
        let book = BookResult::new("Test Book".to_string(), words);
        let entries = aggregate(&[book]);

        let hello = entries.iter().find(|e| e.word == "hello").unwrap();
        assert_eq!(hello.sources.len(), 1);
        assert_eq!(hello.sources[0].book, "Test Book");
        assert_eq!(hello.sources[0].count, 2);
        // frequency_per_1k = 2/4 * 1000 = 500
        assert!((hello.frequency_per_1k - 500.0).abs() < 0.01);
        assert_eq!(hello.difficulty, "easy");
    }

    #[test]
    fn test_aggregate_two_books() {
        let b1 = BookResult::new(
            "Book A".to_string(),
            vec!["hello".to_string(), "world".to_string()],
        );
        let b2 = BookResult::new(
            "Book B".to_string(),
            vec!["hello".to_string(), "there".to_string()],
        );
        let entries = aggregate(&[b1, b2]);

        let hello = entries.iter().find(|e| e.word == "hello").unwrap();
        assert_eq!(hello.sources.len(), 2);
        // total_corpus = 4 words, hello count = 2 → 500 per 1k
        assert!((hello.frequency_per_1k - 500.0).abs() < 0.01);
    }
}
