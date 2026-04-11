use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// ── Parquet ────────────────────────────────────────────────────────────────

/// Emits words from a parquet file using the Arrow batch reader.
///
/// Only string columns (Utf8 / LargeUtf8) are decoded; all other columns are
/// projected away at the reader level so they never enter memory.  Progress is
/// reported proportionally to rows decoded vs. total rows in the file.
pub fn extract_words_from_parquet(
    path: &Path,
    mut on_word: impl FnMut(&str),
    mut on_bytes: impl FnMut(u64),
) {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use parquet::arrow::ProjectionMask;
    use arrow_array::{Array, LargeStringArray, StringArray};
    use arrow_schema::DataType;

    let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Warning: skipping Parquet {:?}: {}", path, e);
            return;
        }
    };

    let builder = match ParquetRecordBatchReaderBuilder::try_new(file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Warning: skipping Parquet {:?}: {}", path, e);
            return;
        }
    };

    // Project only string-typed leaf columns.
    let arrow_schema = builder.schema().clone();
    let parquet_schema = builder.parquet_schema();
    let string_col_indices: Vec<usize> = arrow_schema
        .fields()
        .iter()
        .enumerate()
        .filter(|(_, f)| matches!(f.data_type(), DataType::Utf8 | DataType::LargeUtf8))
        .map(|(i, _)| i)
        .collect();

    let mask = ProjectionMask::leaves(parquet_schema, string_col_indices);

    let total_rows: u64 = builder
        .metadata()
        .row_groups()
        .iter()
        .map(|rg| rg.num_rows().max(0) as u64)
        .sum::<u64>()
        .max(1);

    let reader = match builder.with_projection(mask).with_batch_size(4096).build() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Warning: skipping Parquet {:?}: {}", path, e);
            return;
        }
    };

    let mut rows_done: u64 = 0;
    let mut bytes_reported: u64 = 0;

    // Columns detected as text content (set after the first batch).
    // A column qualifies if any value in the first batch has >= MIN_WORDS_FOR_TEXT
    // words.  This filters out metadata columns (booleans stored as "false"/"true",
    // IDs, timestamps, scores) without hard-coding column names.
    const MIN_WORDS_FOR_TEXT: usize = 5;
    let mut text_col_mask: Option<Vec<bool>> = None;

    for batch_result in reader {
        let batch = match batch_result {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Warning: skipping batch in {:?}: {}", path, e);
                continue;
            }
        };

        // On the first batch, decide which columns look like natural-language text.
        if text_col_mask.is_none() {
            let mask: Vec<bool> = batch.columns().iter().map(|col| {
                let check_arr = |arr: &StringArray| -> bool {
                    arr.iter().flatten().any(|v| v.split_whitespace().nth(MIN_WORDS_FOR_TEXT - 1).is_some())
                };
                let check_large = |arr: &LargeStringArray| -> bool {
                    arr.iter().flatten().any(|v| v.split_whitespace().nth(MIN_WORDS_FOR_TEXT - 1).is_some())
                };
                if let Some(arr) = col.as_any().downcast_ref::<StringArray>() {
                    check_arr(arr)
                } else if let Some(arr) = col.as_any().downcast_ref::<LargeStringArray>() {
                    check_large(arr)
                } else {
                    false
                }
            }).collect();
            text_col_mask = Some(mask);
        }

        let col_mask = text_col_mask.as_deref().unwrap();

        for (col_idx, col) in batch.columns().iter().enumerate() {
            if !col_mask.get(col_idx).copied().unwrap_or(false) {
                continue;
            }
            if let Some(arr) = col.as_any().downcast_ref::<StringArray>() {
                for i in 0..arr.len() {
                    if arr.is_valid(i) {
                        for word in arr.value(i).split_whitespace() {
                            on_word(word);
                        }
                    }
                }
            } else if let Some(arr) = col.as_any().downcast_ref::<LargeStringArray>() {
                for i in 0..arr.len() {
                    if arr.is_valid(i) {
                        for word in arr.value(i).split_whitespace() {
                            on_word(word);
                        }
                    }
                }
            }
        }

        rows_done += batch.num_rows() as u64;
        let new_reported = (rows_done * file_size) / total_rows;
        let tick = new_reported.saturating_sub(bytes_reported);
        if tick > 0 {
            on_bytes(tick);
            bytes_reported = new_reported;
        }
    }
    // Any rounding remainder is topped up by the caller in main.rs.
}

// ── JSON ──────────────────────────────────────────────────────────────────

/// Wraps a reader and flushes byte counts to a callback every ~1 MiB.
struct CountingReader<R: Read, F: FnMut(u64)> {
    inner: R,
    on_bytes: F,
    pending: u64,
}

const FLUSH_THRESHOLD: u64 = 1 << 20; // 1 MiB

impl<R: Read, F: FnMut(u64)> Read for CountingReader<R, F> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.pending += n as u64;
        if self.pending >= FLUSH_THRESHOLD {
            (self.on_bytes)(self.pending);
            self.pending = 0;
        }
        Ok(n)
    }
}

/// SAX-walks a JSON document and emits whitespace-split tokens from every
/// string value.  No intermediate `Value` tree is built; memory use is
/// O(max-string-length + parser-stack-depth) regardless of file size.
fn walk_json_strings<R: Read>(
    reader: &mut struson::reader::JsonStreamReader<R>,
    on_word: &mut impl FnMut(&str),
) {
    use struson::reader::{JsonReader, ValueType};

    let vt = match reader.peek() {
        Ok(v) => v,
        Err(_) => return,
    };
    match vt {
        ValueType::String => {
            if let Ok(s) = reader.next_string() {
                for word in s.split_whitespace() {
                    on_word(word);
                }
            }
        }
        ValueType::Array => {
            if reader.begin_array().is_err() { return; }
            while reader.has_next().unwrap_or(false) {
                walk_json_strings(reader, on_word);
            }
            let _ = reader.end_array();
        }
        ValueType::Object => {
            if reader.begin_object().is_err() { return; }
            while reader.has_next().unwrap_or(false) {
                let _ = reader.next_name(); // key — not needed
                walk_json_strings(reader, on_word);
            }
            let _ = reader.end_object();
        }
        _ => { let _ = reader.skip_value(); }
    }
}

/// Emits words from a JSON file (any shape — object, array, NDJSON) using a
/// SAX-style streaming parser.  Ticks `on_bytes` roughly every 1 MiB of the
/// on-disk file consumed; unflushed trailing bytes are topped up by the caller.
pub fn extract_words_from_json(
    path: &Path,
    mut on_word: impl FnMut(&str),
    on_bytes: impl FnMut(u64),
) {
    use struson::reader::{JsonReader, JsonStreamReader};

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Warning: skipping JSON {:?}: {}", path, e);
            return;
        }
    };

    let counting = CountingReader {
        inner: BufReader::new(file),
        on_bytes,
        pending: 0,
    };

    // Enable multiple-top-level-value mode so NDJSON files are handled and
    // so that peek() returns an error (rather than panicking) at EOF.
    use struson::reader::ReaderSettings;
    let settings = ReaderSettings {
        allow_multiple_top_level: true,
        ..Default::default()
    };
    let mut json_reader = JsonStreamReader::new_custom(counting, settings);

    // A file may contain a single top-level value (object or array) or
    // multiple newline-delimited top-level values (NDJSON).
    loop {
        match json_reader.peek() {
            Ok(_) => {}
            Err(_) => break, // EOF — stop cleanly.
        }
        walk_json_strings(&mut json_reader, &mut on_word);
    }
}
