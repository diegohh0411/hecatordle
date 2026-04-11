mod dictionary;
mod extract;
mod frequency;
mod supabase;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use dictionary::download_dictionary;
use extract::{extract_words_from_json, extract_words_from_parquet};
use frequency::FrequencyCounter;
use supabase::upsert_to_supabase;

const CORPUS_PATH: &str = "corpus";
const OUTPUT_PATH: &str = "word_bank.json";
const EASY_FRACTION: f64 = 0.40;
const NORMAL_FRACTION: f64 = 0.80;

/// Clean a raw token into a 5-letter lowercase word using a stack buffer.
/// Returns Some(slice) if the cleaned word is exactly 5 ascii-alpha chars.
#[inline]
fn clean_word(word: &str) -> Option<[u8; 5]> {
    let mut buf = [0u8; 5];
    let mut len = 0usize;
    for &ch in word.as_bytes() {
        if ch.is_ascii_alphabetic() {
            if len >= 5 { return None; }
            buf[len] = ch.to_ascii_lowercase();
            len += 1;
        }
    }
    if len == 5 { Some(buf) } else { None }
}

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
    let dictionary = Arc::new(dictionary);

    let file_entries: Vec<_> = WalkDir::new(CORPUS_PATH)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !matches!(ext, "json" | "parquet") {
                return false;
            }
            // Skip unresolved LFS pointer files (always ~134 bytes)
            path.metadata().map(|m| m.len() > 512).unwrap_or(false)
        })
        .collect();

    let total_bytes: u64 = file_entries.iter()
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum();

    let pb = ProgressBar::new(total_bytes);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let counters: Vec<FrequencyCounter> = file_entries.par_iter().map(|entry| {
        let path = entry.path();
        let dict = &dictionary;
        let mut local = FrequencyCounter::new();

        let mut on_word = |word: &str| {
            if let Some(buf) = clean_word(word) {
                let clean = std::str::from_utf8(&buf).unwrap();
                if dict.contains(clean) {
                    local.add_str(clean);
                }
            }
        };

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        match extension {
            "parquet" => extract_words_from_parquet(path, &mut on_word),
            "json"    => extract_words_from_json(path, &mut on_word),
            _         => {}
        }

        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        pb.inc(file_size);
        local
    }).collect();

    pb.finish_with_message("Corpus processing complete.");

    let counter = counters.into_iter()
        .reduce(FrequencyCounter::merge)
        .unwrap_or_else(FrequencyCounter::new);

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
