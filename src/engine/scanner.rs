use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::parser::{Record, parse_line};

pub fn scan_line(line: &str) -> Record {
    parse_line(line)
}

pub fn scan_file_lines(
    path: impl AsRef<Path>,
) -> io::Result<impl Iterator<Item = io::Result<Record>>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file)
        .lines()
        .map(|line| line.map(|line| scan_line(&line))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_without_dropping_raw_line() {
        let record = scan_line("DEBUG scan me");

        assert_eq!(record.raw, "DEBUG scan me");
    }
}
