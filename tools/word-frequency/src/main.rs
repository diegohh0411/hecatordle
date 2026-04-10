mod dictionary;
mod extract;
mod frequency;
mod supabase;

use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use indicatif::{ProgressBar, ProgressStyle};

use dictionary::download_dictionary;
use extract::{extract_text_from_json, extract_text_from_parquet, extract_text_from_pdf};
use frequency::FrequencyCounter;
use supabase::upsert_to_supabase;

const CORPUS_PATH: &str = "corpus";
const OUTPUT_PATH: &str = "word_bank.json";
const EASY_FRACTION: f64 = 0.40;
const NORMAL_FRACTION: f64 = 0.80;

fn main() {
    if !Path::new(CORPUS_PATH).exists() {
        println!("Please create a '{}' directory with text files.", CORPUS_PATH);
        return;
    }

    let dictionary = download_dictionary();
    if dictionary.is_empty() {
        println!("Failed to load dictionary.");
        return;
    }

    let file_entries: Vec<_> = WalkDir::new(CORPUS_PATH)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    let pb = ProgressBar::new(file_entries.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let mut counter = FrequencyCounter::new();
    let re = Regex::new(r"^[a-z]{5}$").unwrap();

    for entry in file_entries {
        let path = entry.path();
        pb.set_message(format!("Processing: {:?}", path.file_name().unwrap_or_default()));

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let content = match extension {
            "pdf"     => extract_text_from_pdf(path),
            "parquet" => extract_text_from_parquet(path),
            "json"    => extract_text_from_json(path),
            _ => fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("Warning: skipping {:?}: {}", path, e);
                String::new()
            }),
        };

        for word in content.split_whitespace() {
            let clean_word = word.to_lowercase()
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();
            if re.is_match(&clean_word) && dictionary.contains(&clean_word) {
                counter.add(clean_word);
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("Corpus processing complete.");

    let results = counter.build(EASY_FRACTION, NORMAL_FRACTION);

    let json = serde_json::to_string_pretty(&results).unwrap();
    fs::write(OUTPUT_PATH, &json).expect("Unable to write output file");
    println!("Processed {} unique words. Results saved to {}", results.len(), OUTPUT_PATH);

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
