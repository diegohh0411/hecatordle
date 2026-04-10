use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct WordData {
    word: String,
    count: usize,
    frequency_per_1k: f64,
    difficulty: String,
}

fn main() {
    let corpus_path = "corpus";
    if !Path::new(corpus_path).exists() {
        println!("Please create a 'corpus' directory with text files.");
        return;
    }

    let mut word_counts: HashMap<String, usize> = HashMap::new();
    let mut total_words = 0;
    let re = Regex::new(r"^[a-z]{5}$").unwrap();

    for entry in WalkDir::new(corpus_path).into_entries().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            for word in content.split_whitespace() {
                let clean_word = word.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect::<String>();
                if re.is_match(&clean_word) {
                    *word_counts.entry(clean_word).or_insert(0) += 1;
                    total_words += 1;
                }
            }
        }
    }

    let mut results: Vec<WordData> = word_counts.into_iter().map(|(word, count)| {
        let freq = (count as f64 / total_words as f64) * 1000.0;
        let difficulty = if freq > 1.0 {
            "easy"
        } else if freq > 0.1 {
            "normal"
        } else {
            "hard"
        };
        WordData { word, count, frequency_per_1k: freq, difficulty: difficulty.to_string() }
    }).collect();

    results.sort_by(|a, b| b.count.cmp(&a.count));

    let json = serde_json::to_string_pretty(&results).unwrap();
    fs::write("word_bank.json", json).expect("Unable to write file");
    println!("Processed {} words. Results saved to word_bank.json", total_words);
}
