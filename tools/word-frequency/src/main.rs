use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::path::Path;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use walkdir::WalkDir;
use regex::Regex;
use serde::{Serialize, Deserialize};
use indicatif::{ProgressBar, ProgressStyle};
use dotenv::dotenv;
use lopdf::Document;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WordData {
    word: String,
    frequency_per_1k: f64,
    difficulty: String,
}

fn upsert_to_supabase(results: &[WordData]) {
    dotenv().ok();
    let url = std::env::var("SUPABASE_URL").ok();
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();

    if let (Some(url), Some(key)) = (url, key) {
        let client = reqwest::blocking::Client::new();
        let endpoint = format!("{}/rest/v1/word_bank", url);

        // Delete all existing rows before re-inserting
        println!("Clearing existing word_bank table...");
        let del = client.delete(format!("{}?word=not.is.null", endpoint))
            .header("apikey", &key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Prefer", "return=minimal")
            .send();
        match del {
            Ok(resp) if !resp.status().is_success() => {
                println!("Error clearing table: {:?}", resp.text());
                return;
            }
            Err(e) => {
                println!("Failed to clear table: {}", e);
                return;
            }
            _ => println!("Table cleared."),
        }

        println!("Inserting {} words...", results.len());
        for chunk in results.chunks(1000) {
            let res = client.post(&endpoint)
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .header("Prefer", "return=minimal")
                .json(chunk)
                .send();

            match res {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        println!("Error inserting chunk: {:?}", resp.text());
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

fn collect_json_strings(value: &serde_json::Value, text: &mut String) {
    match value {
        serde_json::Value::String(s) => { text.push_str(s); text.push(' '); }
        serde_json::Value::Array(arr) => { for v in arr { collect_json_strings(v, text); } }
        serde_json::Value::Object(map) => { for v in map.values() { collect_json_strings(v, text); } }
        _ => {}
    }
}

fn extract_text_from_json(path: &Path) -> String {
    let mut text = String::new();
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(value) = serde_json::from_str(&content) {
            collect_json_strings(&value, &mut text);
        }
    }
    text
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

fn extract_text_from_parquet(path: &Path) -> String {
    let mut text = String::new();

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return text,
    };
    let reader = match SerializedFileReader::new(file) {
        Ok(r) => r,
        Err(_) => return text,
    };
    let row_iter = match reader.get_row_iter(None) {
        Ok(iter) => iter,
        Err(_) => return text,
    };

    for row_result in row_iter {
        let row = match row_result {
            Ok(r) => r,
            Err(_) => continue,
        };
        for (_, field) in row.get_column_iter() {
            match field {
                Field::Str(s) => { text.push_str(s); text.push(' '); }
                Field::Bytes(b) => {
                    text.push_str(&String::from_utf8_lossy(b.data()));
                    text.push(' ');
                }
                _ => {}
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
        let content = match extension {
            "pdf"     => extract_text_from_pdf(path),
            "parquet" => extract_text_from_parquet(path),
            "json"    => extract_text_from_json(path),
            _         => fs::read_to_string(path).unwrap_or_default(),
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
        let frequency_per_1k = (count as f64 / total_words as f64) * 1000.0;
        WordData { word, frequency_per_1k, difficulty: String::new() }
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

    print!("Push {} words to Supabase? [y/N] ", results.len());
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim().eq_ignore_ascii_case("y") {
        upsert_to_supabase(&results);
    } else {
        println!("Skipping Supabase sync.");
    }
}
