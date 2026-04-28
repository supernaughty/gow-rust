//! Lazy line-offset index for the `less` pager.
//!
//! Stores `Vec<u64>` of byte offsets, one per line start. Builds incrementally
//! as the user scrolls forward. Never loads the full file content into memory
//! — only the offset table grows (~8 bytes per line).
//!
//! Implements decision D-09: streaming I/O, no full-file buffering.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};

/// Lazy line-offset index over a seekable file.
pub struct LineIndex {
    /// Byte offsets where each line starts. `offsets[i]` is the start of line `i`.
    /// After scanning past EOF, the final entry is the file's total length.
    offsets: Vec<u64>,
    reader: BufReader<File>,
    eof_reached: bool,
}

impl LineIndex {
    /// Create a new `LineIndex` from an open `File`.
    ///
    /// The file must be seekable. The initial state is: `offsets = [0]` (line 0
    /// starts at byte 0) and `eof_reached = false`.
    pub fn new(file: File) -> Self {
        Self {
            offsets: vec![0],
            reader: BufReader::new(file),
            eof_reached: false,
        }
    }

    /// Read forward until line `line_num` is indexed, or EOF is reached.
    ///
    /// After calling this, `offsets.len() > line_num` OR `eof_reached == true`.
    pub fn ensure_indexed_to(&mut self, line_num: usize) -> io::Result<()> {
        // If we already have the offset we need, do nothing.
        if self.offsets.len() > line_num || self.eof_reached {
            return Ok(());
        }
        // Seek to the last known offset so the reader is in sync.
        let last_known = *self.offsets.last().unwrap();
        self.reader.seek(SeekFrom::Start(last_known))?;
        while self.offsets.len() <= line_num && !self.eof_reached {
            let mut buf = Vec::new();
            let n = self.reader.read_until(b'\n', &mut buf)?;
            if n == 0 {
                self.eof_reached = true;
                break;
            }
            let next_offset = self.offsets.last().unwrap() + n as u64;
            self.offsets.push(next_offset);
        }
        Ok(())
    }

    /// Scan the entire file to EOF, returning the total number of lines.
    ///
    /// After this call `is_complete()` returns `true`.
    /// The return value is `line_count_so_far()` after completing the scan.
    pub fn scan_to_end(&mut self) -> io::Result<usize> {
        if !self.eof_reached {
            // Seek to the last indexed position before scanning forward.
            let last_known = *self.offsets.last().unwrap();
            self.reader.seek(SeekFrom::Start(last_known))?;
            while !self.eof_reached {
                let mut buf = Vec::new();
                let n = self.reader.read_until(b'\n', &mut buf)?;
                if n == 0 {
                    self.eof_reached = true;
                    break;
                }
                let next_offset = self.offsets.last().unwrap() + n as u64;
                self.offsets.push(next_offset);
            }
        }
        Ok(self.line_count_so_far())
    }

    /// Number of lines currently indexed.
    ///
    /// Does NOT include the trailing EOF-position entry that `scan_to_end`
    /// appends. So for a 3-line file that has been fully scanned:
    /// `offsets = [0, 6, 11, 17, 17_or_len]` → 3 lines.
    pub fn line_count_so_far(&self) -> usize {
        // `offsets` always starts with [0]. After scanning each line, we push
        // the start of the *next* line (= end of current line). When EOF is
        // reached without finding a newline, the final push is not performed
        // (loop terminates with `n == 0`).
        //
        // Invariant: offsets.len() - 1 == number of lines scanned.
        // If offsets.len() == 1 and eof has been reached, file is empty.
        if self.offsets.len() == 0 {
            return 0;
        }
        // Each entry after the first [0] represents one complete line read.
        // Number of lines = offsets.len() - 1.
        // For an empty file: scan_to_end reads 0 bytes, eof_reached=true, offsets=[0].
        // => line_count = 0. Correct.
        // For "hello" (no newline): read_until returns 5 bytes, pushes offset 5.
        // offsets=[0,5], eof_reached=true on next read. line_count=1. Correct.
        self.offsets.len() - 1
    }

    /// Returns `true` once the file has been scanned to EOF.
    pub fn is_complete(&self) -> bool {
        self.eof_reached
    }

    /// Read the content of line `line_num` (0-based).
    ///
    /// Returns `Ok(None)` if the line is past the end of the file.
    /// The returned bytes include the trailing `\n` if present; callers decide
    /// whether to strip it.
    ///
    /// Internally seeks to the correct byte offset so random access works even
    /// after `scan_to_end`.
    pub fn read_line_at(&mut self, line_num: usize) -> io::Result<Option<Vec<u8>>> {
        // Make sure we have indexed at least up to line_num + 1 so we know its end offset.
        self.ensure_indexed_to(line_num + 1)?;
        if line_num >= self.line_count_so_far() {
            return Ok(None);
        }
        let start = self.offsets[line_num];
        self.reader.seek(SeekFrom::Start(start))?;
        let mut buf = Vec::new();
        self.reader.read_until(b'\n', &mut buf)?;
        Ok(Some(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_file(content: &[u8]) -> File {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content).unwrap();
        tmp.flush().unwrap();
        // Reopen for reading from start (don't reuse the writer handle position).
        File::open(tmp.path()).unwrap()
    }

    /// Test 1: Empty file → 0 lines, is_complete after scan.
    #[test]
    fn test_empty_file_zero_lines() {
        let f = make_file(b"");
        let mut idx = LineIndex::new(f);
        let total = idx.scan_to_end().unwrap();
        assert_eq!(total, 0, "empty file should have 0 lines");
        assert!(idx.is_complete(), "empty file should be complete after scan");
        assert_eq!(
            idx.read_line_at(0).unwrap(),
            None,
            "line 0 of empty file should be None"
        );
    }

    /// Test 2: Single line without trailing newline.
    #[test]
    fn test_single_line_no_newline() {
        let f = make_file(b"hello");
        let mut idx = LineIndex::new(f);
        assert_eq!(idx.scan_to_end().unwrap(), 1, "single line = 1");
        assert_eq!(
            idx.read_line_at(0).unwrap(),
            Some(b"hello".to_vec()),
            "content mismatch"
        );
        assert_eq!(
            idx.read_line_at(1).unwrap(),
            None,
            "past-EOF should be None"
        );
    }

    /// Test 3: Three lines with trailing newline — verify all lines readable.
    #[test]
    fn test_three_lines() {
        let f = make_file(b"alpha\nbeta\ngamma\n");
        let mut idx = LineIndex::new(f);
        assert_eq!(idx.scan_to_end().unwrap(), 3);
        assert_eq!(
            idx.read_line_at(0).unwrap(),
            Some(b"alpha\n".to_vec())
        );
        assert_eq!(
            idx.read_line_at(1).unwrap(),
            Some(b"beta\n".to_vec())
        );
        assert_eq!(
            idx.read_line_at(2).unwrap(),
            Some(b"gamma\n".to_vec())
        );
    }

    /// Test 4: Lazy indexing — `ensure_indexed_to(2)` should NOT scan the full file (D-09).
    #[test]
    fn test_lazy_indexing_does_not_scan_full_file() {
        let f = make_file(b"line1\nline2\nline3\nline4\nline5\n");
        let mut idx = LineIndex::new(f);
        // Ask for line 2 (0-based = third line).
        idx.ensure_indexed_to(2).unwrap();
        // We should have at least 3 offsets (lines 0, 1, 2 scanned).
        assert!(
            idx.line_count_so_far() >= 2,
            "should have at least 2 lines indexed, got {}",
            idx.line_count_so_far()
        );
        // Should NOT have reached EOF yet (5 lines, only asked for 3).
        assert!(
            !idx.is_complete(),
            "should NOT have hit EOF after indexing only 3 of 5 lines"
        );
    }

    /// Test 5: Seek backward — after a forward scan, re-reading earlier lines works correctly.
    #[test]
    fn test_seek_back_after_forward_scan() {
        let f = make_file(b"a\nbb\nccc\n");
        let mut idx = LineIndex::new(f);
        idx.scan_to_end().unwrap();
        // Re-read line 0 after scanning past it — verify seek correctness.
        assert_eq!(
            idx.read_line_at(0).unwrap(),
            Some(b"a\n".to_vec()),
            "line 0 seek-back failed"
        );
        assert_eq!(
            idx.read_line_at(2).unwrap(),
            Some(b"ccc\n".to_vec()),
            "line 2 seek-back failed"
        );
    }

    /// Test 6: Unicode byte offsets — multi-byte UTF-8 chars must be byte-counted (critical for Korean).
    #[test]
    fn test_unicode_byte_offsets_correct() {
        // Korean text: "한글" = 6 bytes (3 bytes each), "파이팅" = 9 bytes (3 bytes each).
        let content = "한글\n파이팅\n".as_bytes();
        let f = make_file(content);
        let mut idx = LineIndex::new(f);
        assert_eq!(
            idx.scan_to_end().unwrap(),
            2,
            "should find 2 lines"
        );
        assert_eq!(
            idx.read_line_at(0).unwrap().unwrap(),
            "한글\n".as_bytes(),
            "line 0 byte content mismatch"
        );
        assert_eq!(
            idx.read_line_at(1).unwrap().unwrap(),
            "파이팅\n".as_bytes(),
            "line 1 byte content mismatch"
        );
    }
}
