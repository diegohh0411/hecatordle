use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;
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

fn extract_text_from_pdf(path: &Path) -> String {
    println!("Extracting text from PDF: {:?}", path.display());
    
    // Method 1: Use pdf-extract (best for retail PDFs with text layers)
    if let Ok(text) = pdf_extract::extract_text(path) {
        if text.trim().len() > 100 {
            return text;
        }
    }

    // Method 2: Fallback to Tesseract (if the PDF is scanned images)
    // Note: This requires converting PDF to images first. 
    // Since we don't have pdftoppm, we'll suggest it to the user if Method 1 fails.
    println!("Warning: Could not extract sufficient text layer from PDF. Scanned PDFs are not yet supported without poppler-utils.");
    String::new()
}

fn download_dictionary() -> HashSet<String> {
    const DICT_URL: &str = "https://raw.githubusercontent.com/Kinkelin/WordleCompetition/main/data/official/official_allowed_guesses.txt";
    const SOLUTIONS_URL: &str = "https://raw.githubusercontent.com/Kinkelin/WordleCompetition/main/data/official/shuffled_real_wordles.txt";
    
    println!("Downloading validation dictionaries...");
    let mut dict = HashSet::new();
    
    for url in [DICT_URL, SOLUTIONS_URL] {
        if let Ok(resp) = reqwest::blocking::get(url) {
            if let Ok(text) = resp.text() {
                for line in text.lines() {
                    let word = line.trim().to_lowercase();
                    if word.len() == 5 {
                        dict.insert(word);
                    }
                }
            }
        }
    }
    println!("Loaded {} valid 5-letter words for validation.", dict.len());
    dict
}

fn main() {
    let corpus_path = "corpus";
    if !Path::new(corpus_path).exists() {
        println!("Please create a 'corpus' directory with text files.");
        return;
    }

    let dictionary = download_dictionary();
    if dictionary.is_empty() {
        println!("Failed to load dictionary. Please check your internet connection.");
        return;
    }

    let mut word_counts: HashMap<String, usize> = HashMap::new();
    let mut total_words = 0;
    let re = Regex::new(r"^[a-z]{5}$").unwrap();

    for entry in WalkDir::new(corpus_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            let extension = entry.path().extension().and_then(|s| s.to_str()).unwrap_or("");
            let content = if extension == "pdf" {
                extract_text_from_pdf(entry.path())
            } else {
                fs::read_to_string(entry.path()).unwrap_or_default()
            };

            for word in content.split_whitespace() {
                let clean_word = word.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect::<String>();
                if re.is_match(&clean_word) && dictionary.contains(&clean_word) {
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
