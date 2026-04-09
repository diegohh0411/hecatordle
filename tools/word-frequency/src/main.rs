mod frequency;
mod output;
mod parser;

use std::fs;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};

use frequency::BookResult;

/// Calculate word frequencies from book corpora and export for Hecatordle.
///
/// Accepts plain text files directly, or image/PDF files via Tesseract OCR.
/// Outputs JSON (for Supabase word_bank import), CSV, or TypeScript (word-list.ts).
///
/// Examples:
///   word-frequency --input frankenstein.txt dracula.txt --format json
///   word-frequency --input scan.pdf --format typescript
///   word-frequency --input *.txt --format csv --output words.csv
#[derive(Parser)]
#[command(name = "word-frequency", version, about, long_about = None)]
struct Cli {
    /// Input files to process (.txt for plain text, .pdf/.png/.jpg/.tif for OCR)
    #[arg(short, long, num_args = 1.., required = true)]
    input: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    format: Format,

    /// Output file path (default: stdout for json/csv, src/game/word-list.ts for typescript)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Minimum frequency per 1k words to include a word (default: 0, include all)
    #[arg(long, default_value_t = 0.0)]
    min_frequency: f64,

    /// Maximum number of words to include (0 = unlimited)
    #[arg(long, default_value_t = 0)]
    limit: usize,
}

#[derive(Clone, ValueEnum)]
enum Format {
    Json,
    Csv,
    Typescript,
}

fn main() {
    let cli = Cli::parse();

    let mut books: Vec<BookResult> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for path in &cli.input {
        match process_file(path) {
            Ok(book) => {
                eprintln!(
                    "Processed {:?}: {} total words, {} unique 5-letter words",
                    path,
                    book.total_words,
                    book.word_counts.len()
                );
                books.push(book);
            }
            Err(e) => {
                errors.push(format!("{:?}: {}", path, e));
            }
        }
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("Error: {err}");
        }
        if books.is_empty() {
            std::process::exit(1);
        }
    }

    let mut entries = frequency::aggregate(&books);

    if cli.min_frequency > 0.0 {
        entries.retain(|e| e.frequency_per_1k >= cli.min_frequency);
    }

    if cli.limit > 0 && entries.len() > cli.limit {
        entries.truncate(cli.limit);
    }

    eprintln!(
        "Total: {} words ({} easy, {} normal, {} hard)",
        entries.len(),
        entries.iter().filter(|e| e.difficulty == "easy").count(),
        entries.iter().filter(|e| e.difficulty == "normal").count(),
        entries.iter().filter(|e| e.difficulty == "hard").count(),
    );

    let result = match &cli.output {
        Some(path) => write_to_file(&entries, &cli.format, path),
        None => {
            let format_str = match cli.format {
                Format::Typescript => "typescript",
                Format::Json => "json",
                Format::Csv => "csv",
            };
            match output::resolve_output_path(None, format_str) {
                Some(default_path) => write_to_file(&entries, &cli.format, &default_path),
                None => write_to_stdout(&entries, &cli.format),
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Error writing output: {e}");
        std::process::exit(1);
    }
}

fn process_file(path: &Path) -> Result<BookResult, String> {
    let book_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let text = if parser::needs_ocr(path) {
        parser::ocr_file(path)?
    } else {
        fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {e}"))?
    };

    let words = parser::extract_words_from_text(&text);
    Ok(BookResult::new(book_name, words))
}

fn write_to_file(
    entries: &[frequency::WordEntry],
    format: &Format,
    path: &Path,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {e}"))?;
        }
    }
    let file =
        fs::File::create(path).map_err(|e| format!("Failed to create output file: {e}"))?;
    let writer = BufWriter::new(file);
    dispatch_write(entries, format, writer)?;
    eprintln!("Written to {:?}", path);
    Ok(())
}

fn write_to_stdout(entries: &[frequency::WordEntry], format: &Format) -> Result<(), String> {
    let stdout = io::stdout();
    let writer = BufWriter::new(stdout.lock());
    dispatch_write(entries, format, writer)
}

fn dispatch_write<W: io::Write>(
    entries: &[frequency::WordEntry],
    format: &Format,
    writer: W,
) -> Result<(), String> {
    match format {
        Format::Json => output::write_json(entries, writer),
        Format::Csv => output::write_csv(entries, writer),
        Format::Typescript => output::write_typescript(entries, writer),
    }
}
