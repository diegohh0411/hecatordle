use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use serde::{Serialize, Deserialize};
use indicatif::{ProgressBar, ProgressStyle};
use dotenv::dotenv;
use lopdf::Document;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WordData {
    word: String,
    #[serde(skip_serializing)]
    count: usize,
    frequency_per_1k: f64,
    difficulty: String,
}

fn upsert_to_supabase(results: &[WordData]) {
    dotenv().ok();
    let url = std::env::var("SUPABASE_URL").ok();
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();

    if let (Some(url), Some(key)) = (url, key) {
        println!("Pushing {} words to Supabase...", results.len());
        let client = reqwest::blocking::Client::new();
        let endpoint = format!("{}/rest/v1/word_bank", url);
        
        for chunk in results.chunks(1000) {
            let res = client.post(&endpoint)
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .header("Prefer", "resolution=merge-duplicates")
                .json(chunk)
                .send();

            match res {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        println!("Error uploading chunk: {:?}", resp.text());
                    }
                },
                Err(e) => println!("Request failed: {}", e),
            }
        }
        println!("Supabase sync complete.");
    } else {
        println!("Skipping Supabase sync: SUPABASE_URL or SUPABASE_SERVICE_ROLE_KEY not found in .env");
    }
}

fn extract_text_from_pdf(path: &Path) -> String {
    let mut text = String::new();
    if let Ok(doc) = Document::load(path) {
        let pages = doc.get_pages();
        for (&page_num, _) in &pages {
            if let Ok(page_text) = doc.extract_text(&[page_num]) {
                text.push_str(&page_text);
                text.push(' ');
            }
        }
    }
    text
}

fn download_dictionary() -> HashSet<String> {
    const DICT_URL: &str = "https://raw.githubusercontent.com/Kinkelin/WordleCompetition/main/data/official/official_allowed_guesses.txt";
    const SOLUTIONS_URL: &str = "https://raw.githubusercontent.com/Kinkelin/WordleCompetition/main/data/official/shuffled_real_wordles.txt";
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap());
    pb.set_message("Downloading validation dictionaries...");
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    let mut dict = HashSet::new();
    for url in [DICT_URL, SOLUTIONS_URL] {
        if let Ok(resp) = reqwest::blocking::get(url) {
            if let Ok(content) = resp.text() {
                for line in content.lines() {
                    let word = line.trim().to_lowercase();
                    if word.len() == 5 {
                        dict.insert(word);
                    }
                }
            }
        }
    }
    pb.finish_with_message(format!("Loaded {} valid 5-letter words.", dict.len()));
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
        println!("Failed to load dictionary.");
        return;
    }

    let file_entries: Vec<_> = WalkDir::new(corpus_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    let pb = ProgressBar::new(file_entries.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut word_counts: HashMap<String, usize> = HashMap::new();
    let mut total_words = 0;
    let re = Regex::new(r"^[a-z]{5}$").unwrap();

    for entry in file_entries {
        let path = entry.path();
        pb.set_message(format!("Processing: {:?}", path.file_name().unwrap_or_default()));
        
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let content = if extension == "pdf" {
            extract_text_from_pdf(path)
        } else {
            fs::read_to_string(path).unwrap_or_default()
        };

        for word in content.split_whitespace() {
            let clean_word = word.to_lowercase().chars().filter(|c| c.is_alphabetic()).collect::<String>();
            if re.is_match(&clean_word) && dictionary.contains(&clean_word) {
                *word_counts.entry(clean_word).or_insert(0) += 1;
                total_words += 1;
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("Corpus processing complete.");

    let mut results: Vec<WordData> = word_counts.into_iter().map(|(word, count)| {
        let freq = (count as f64 / total_words as f64) * 1000.0;
        WordData { word, count, frequency_per_1k: freq, difficulty: String::new() }
    }).collect();

    results.sort_by(|a, b| b.frequency_per_1k.partial_cmp(&a.frequency_per_1k).unwrap_or(std::cmp::Ordering::Equal));

    let total_unique = results.len();
    let easy_threshold = (total_unique as f64 * 0.4) as usize;
    let normal_threshold = (total_unique as f64 * 0.8) as usize;

    for (i, data) in results.iter_mut().enumerate() {
        if i < easy_threshold {
            data.difficulty = "easy".to_string();
        } else if i < normal_threshold {
            data.difficulty = "normal".to_string();
        } else {
            data.difficulty = "hard".to_string();
        }
    }

    let json = serde_json::to_string_pretty(&results).unwrap();
    fs::write("word_bank.json", json).expect("Unable to write file");
    println!("Processed {} words. Results saved to word_bank.json", total_words);

    upsert_to_supabase(&results);
}
