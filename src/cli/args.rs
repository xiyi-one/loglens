use clap::Parser;

use crate::output::OutputFormat;

#[derive(Debug, Parser)]
#[command(name = "loglens")]
#[command(about = "Local-first natural-language log search")]
pub struct Args {
    /// Files, glob patterns, "-", and, without --dsl, a trailing natural-language query.
    #[arg(value_name = "ARG")]
    pub positionals: Vec<String>,

    /// DSL JSON string or path to a DSL JSON file.
    #[arg(long)]
    pub dsl: Option<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,

    /// Override the DSL result limit.
    #[arg(long)]
    pub limit: Option<usize>,

    /// Continue reading new lines appended to file inputs.
    #[arg(short, long)]
    pub follow: bool,

    /// Explain the planned DSL without hidden behavior.
    #[arg(long)]
    pub explain: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_empty_args() {
        let args = Args::parse_from(["loglens"]);

        assert!(args.dsl.is_none());
        assert!(args.positionals.is_empty());
    }

    #[test]
    fn parses_dsl_inputs_and_output() {
        let args = Args::parse_from([
            "loglens",
            "--dsl",
            r#"{"version":1,"filters":{}}"#,
            "--output",
            "json",
            "--limit",
            "5",
            "--follow",
            "app.log",
            "*.log",
        ]);

        assert_eq!(args.positionals, vec!["app.log", "*.log"]);
        assert_eq!(args.output, OutputFormat::Json);
        assert_eq!(args.limit, Some(5));
        assert!(args.follow);
    }

    #[test]
    fn parses_natural_language_query_position() {
        let args = Args::parse_from(["loglens", "app.log", "login failed"]);

        assert_eq!(args.dsl, None);
        assert_eq!(args.positionals, vec!["app.log", "login failed"]);
    }
}
