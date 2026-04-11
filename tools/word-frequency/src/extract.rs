use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::{Field, Row};

fn emit_row_strings(row: &Row, on_word: &mut impl FnMut(&str)) {
    for (_, field) in row.get_column_iter() {
        match field {
            Field::Str(s) => {
                for word in s.split_whitespace() {
                    on_word(word);
                }
            }
            Field::Bytes(b) => {
                if let Ok(s) = std::str::from_utf8(b.data()) {
                    for word in s.split_whitespace() {
                        on_word(word);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Emits words from a parquet file, ticking `on_bytes` once per row group.
/// Per-row-group ticks are prorated to the on-disk file size so total reported
/// bytes approximate (but may fall short of) `file_size` — the caller tops up.
pub fn extract_words_from_parquet(
    path: &Path,
    mut on_word: impl FnMut(&str),
    mut on_bytes: impl FnMut(u64),
) {
    let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return; }
    };
    let reader = match SerializedFileReader::new(file) {
        Ok(r) => r,
        Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return; }
    };

    let metadata = reader.metadata();
    let num_row_groups = reader.num_row_groups();
    let total_rg_bytes: i64 = (0..num_row_groups)
        .map(|i| metadata.row_group(i).compressed_size())
        .sum();

    if num_row_groups == 0 || total_rg_bytes <= 0 {
        let row_iter = match reader.get_row_iter(None) {
            Ok(iter) => iter,
            Err(e) => { eprintln!("Warning: skipping Parquet {:?}: {}", path, e); return; }
        };
        for row_result in row_iter {
            match row_result {
                Ok(row) => emit_row_strings(&row, &mut on_word),
                Err(e) => eprintln!("Warning: skipping row in {:?}: {}", path, e),
            }
        }
        return;
    }

    let total_rg_bytes = total_rg_bytes as u128;
    let file_size_u128 = file_size as u128;
    let mut reported: u64 = 0;

    for i in 0..num_row_groups {
        let rg_reader = match reader.get_row_group(i) {
            Ok(r) => r,
            Err(e) => { eprintln!("Warning: skipping row group {} in {:?}: {}", i, path, e); continue; }
        };
        let row_iter = match rg_reader.get_row_iter(None) {
            Ok(iter) => iter,
            Err(e) => { eprintln!("Warning: skipping row group {} in {:?}: {}", i, path, e); continue; }
        };
        for row_result in row_iter {
            match row_result {
                Ok(row) => emit_row_strings(&row, &mut on_word),
                Err(e) => eprintln!("Warning: skipping row in {:?}: {}", path, e),
            }
        }

        let is_last = i + 1 == num_row_groups;
        let tick = if is_last {
            file_size.saturating_sub(reported)
        } else {
            let rg_bytes = metadata.row_group(i).compressed_size().max(0) as u128;
            ((rg_bytes * file_size_u128) / total_rg_bytes) as u64
        };
        if tick > 0 {
            on_bytes(tick);
            reported += tick;
        }
    }
}

fn walk_json_strings(value: &serde_json::Value, on_word: &mut impl FnMut(&str)) {
    match value {
        serde_json::Value::String(s) => {
            for word in s.split_whitespace() {
                on_word(word);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr { walk_json_strings(v, on_word); }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() { walk_json_strings(v, on_word); }
        }
        _ => {}
    }
}

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

/// Emits words from a JSON file, ticking `on_bytes` roughly every 1 MiB of
/// the on-disk file consumed. Unflushed trailing bytes at end-of-file are not
/// reported here — the caller tops the counter up to `file_size`.
pub fn extract_words_from_json(
    path: &Path,
    mut on_word: impl FnMut(&str),
    on_bytes: impl FnMut(u64),
) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { eprintln!("Warning: skipping JSON {:?}: {}", path, e); return; }
    };
    let reader = BufReader::new(file);
    let counting = CountingReader {
        inner: reader,
        on_bytes,
        pending: 0,
    };

    let value: serde_json::Value = match serde_json::from_reader(counting) {
        Ok(v) => v,
        Err(e) => { eprintln!("Warning: skipping JSON {:?}: {}", path, e); return; }
    };
    walk_json_strings(&value, &mut on_word);
}
