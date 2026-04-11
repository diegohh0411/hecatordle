use rustc_hash::FxHashSet;
use indicatif::{ProgressBar, ProgressStyle};

const DICT_URL: &str = "https://raw.githubusercontent.com/Kinkelin/WordleCompetition/main/data/official/official_allowed_guesses.txt";
const SOLUTIONS_URL: &str = "https://raw.githubusercontent.com/Kinkelin/WordleCompetition/main/data/official/shuffled_real_wordles.txt";

pub fn download_dictionary() -> FxHashSet<String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap());
    pb.set_message("Downloading validation dictionaries...");
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    let mut dict = FxHashSet::default();
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
