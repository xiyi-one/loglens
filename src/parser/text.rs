use super::Record;

pub fn parse_text_line(line: &str) -> Record {
    let mut record = Record::from_raw(line);
    record.message = Some(line.to_string());
    record
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_original_raw_line() {
        let record = parse_text_line("WARN  original log content");

        assert_eq!(record.raw, "WARN  original log content");
        assert_eq!(
            record.message.as_deref(),
            Some("WARN  original log content")
        );
        assert!(record.fields.is_empty());
    }
}
