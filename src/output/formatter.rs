use crate::engine::Match;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedLine {
    pub line: String,
}

pub fn render_match(matched: &Match) -> RenderedLine {
    RenderedLine {
        line: matched.record.raw.clone(),
    }
}

pub fn format_match(
    matched: &Match,
    output_format: OutputFormat,
) -> Result<String, serde_json::Error> {
    match output_format {
        OutputFormat::Text => Ok(render_match(matched).line),
        OutputFormat::Json => serde_json::to_string(&matched.record),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Record;

    #[test]
    fn renders_raw_line() {
        let matched = Match {
            record: Record::from_raw("ERROR request failed"),
        };

        assert_eq!(render_match(&matched).line, "ERROR request failed");
    }

    #[test]
    fn renders_json_record() {
        let matched = Match {
            record: Record::from_raw("ERROR request failed"),
        };

        let rendered = format_match(&matched, OutputFormat::Json).expect("json should render");

        assert!(rendered.contains(r#""raw":"ERROR request failed""#));
    }
}
