use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use clap::Parser;
use glob::{PatternError, glob};

use crate::dsl::{Query, validate_query};
use crate::engine::{EngineError, Match, execute_file, execute_record};
use crate::output::{OutputFormat, format_match, format_query_plan};
use crate::parser::parse_line;
use crate::translator::translate_heuristic;

use super::Args;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("at least one input is required")]
    MissingInput,
    #[error("natural-language mode requires at least one input and one query")]
    MissingNaturalLanguageQuery,
    #[error("failed to read DSL file {path}: {source}")]
    ReadDsl { path: String, source: io::Error },
    #[error("failed to parse DSL JSON: {source}\nhint: {hint}")]
    ParseDsl {
        source: serde_json::Error,
        hint: &'static str,
    },
    #[error("failed to format JSON output: {0}")]
    FormatRecord(serde_json::Error),
    #[error("invalid DSL query: {0}")]
    InvalidDsl(#[from] crate::dsl::ValidationError),
    #[error("invalid glob pattern {pattern}: {source}")]
    InvalidGlob {
        pattern: String,
        source: PatternError,
    },
    #[error("glob pattern did not match any files: {0}")]
    EmptyGlob(String),
    #[error("glob expansion failed: {0}")]
    Glob(#[from] glob::GlobError),
    #[error("engine execution failed: {0}")]
    Engine(#[from] EngineError),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("follow mode supports file inputs only")]
    FollowStdin,
}

pub fn run() -> Result<(), CliError> {
    let args = Args::parse();
    run_with_io(args, io::stdin().lock(), io::stdout(), io::stderr())
}

fn run_with_io(
    args: Args,
    stdin: impl BufRead,
    stdout: impl Write,
    stderr: impl Write,
) -> Result<(), CliError> {
    let (mut query, inputs) = resolve_query_and_inputs(args.dsl.as_deref(), &args.positionals)?;
    if let Some(limit) = args.limit {
        query.options.limit = Some(limit);
    }
    validate_query(&query)?;

    if args.explain {
        write_explain(stderr, &query)?;
    }

    let inputs = expand_inputs(&inputs)?;
    if args.follow {
        follow_inputs(inputs, &query, args.output, stdout)
    } else {
        execute_inputs(inputs, &query, args.output, stdin, stdout)
    }
}

fn resolve_query_and_inputs(
    dsl: Option<&str>,
    positionals: &[String],
) -> Result<(Query, Vec<String>), CliError> {
    match dsl {
        Some(dsl) => {
            if positionals.is_empty() {
                return Err(CliError::MissingInput);
            }

            Ok((load_query(dsl)?, positionals.to_vec()))
        }
        None => {
            let (query, inputs) = positionals
                .split_last()
                .ok_or(CliError::MissingNaturalLanguageQuery)?;

            if inputs.is_empty() {
                return Err(CliError::MissingInput);
            }

            Ok((translate_heuristic(query).query, inputs.to_vec()))
        }
    }
}

fn load_query(dsl: &str) -> Result<Query, CliError> {
    let json = if Path::new(dsl).exists() {
        fs::read_to_string(dsl).map_err(|source| CliError::ReadDsl {
            path: dsl.to_string(),
            source,
        })?
    } else {
        dsl.to_string()
    };

    Query::from_json(&json).map_err(|source| CliError::ParseDsl {
        source,
        hint: dsl_parse_hint(dsl),
    })
}

fn dsl_parse_hint(dsl: &str) -> &'static str {
    let trimmed = dsl.trim_start();

    if Path::new(dsl).extension().is_some() && !Path::new(dsl).exists() {
        "the --dsl value looks like a file path, but the file does not exist"
    } else if !trimmed.starts_with('{') {
        "pass a JSON object string or a path to a DSL JSON file"
    } else {
        "check that the JSON uses the LogLens DSL shape: {\"version\":1,\"filters\":{...}}"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Input {
    Stdin,
    File(PathBuf),
}

fn expand_inputs(inputs: &[String]) -> Result<Vec<Input>, CliError> {
    let mut expanded = Vec::new();

    for input in inputs {
        if input == "-" {
            expanded.push(Input::Stdin);
        } else if contains_glob_meta(input) {
            let mut matched = false;
            for path in glob(input).map_err(|source| CliError::InvalidGlob {
                pattern: input.clone(),
                source,
            })? {
                expanded.push(Input::File(path?));
                matched = true;
            }

            if !matched {
                return Err(CliError::EmptyGlob(input.clone()));
            }
        } else {
            expanded.push(Input::File(PathBuf::from(input)));
        }
    }

    Ok(expanded)
}

fn contains_glob_meta(input: &str) -> bool {
    input.contains('*') || input.contains('?') || input.contains('[')
}

fn execute_inputs(
    inputs: Vec<Input>,
    query: &Query,
    output_format: OutputFormat,
    stdin: impl BufRead,
    mut stdout: impl Write,
) -> Result<(), CliError> {
    let mut remaining = query.options.limit.unwrap_or(usize::MAX);
    let mut stdin = Some(stdin);

    for input in inputs {
        if remaining == 0 {
            break;
        }

        let mut scoped_query = query.clone();
        scoped_query.options.limit = Some(remaining);

        let matches = match input {
            Input::File(path) => execute_file(path, &scoped_query)?,
            Input::Stdin => {
                let Some(stdin) = stdin.take() else {
                    continue;
                };
                execute_stdin(stdin, &scoped_query)?
            }
        };

        remaining = remaining.saturating_sub(matches.len());
        write_matches(&mut stdout, output_format, query, &matches)?;
    }

    Ok(())
}

struct FollowInput {
    reader: BufReader<File>,
    position: u64,
}

fn follow_inputs(
    inputs: Vec<Input>,
    query: &Query,
    output_format: OutputFormat,
    mut stdout: impl Write,
) -> Result<(), CliError> {
    let mut files = Vec::new();

    for input in inputs {
        match input {
            Input::Stdin => return Err(CliError::FollowStdin),
            Input::File(path) => {
                let mut reader = BufReader::new(File::open(path)?);
                let position = reader.seek(SeekFrom::End(0))?;
                files.push(FollowInput { reader, position });
            }
        }
    }

    loop {
        let mut saw_new_data = false;

        for file in &mut files {
            let matches = read_appended_matches(file, query)?;
            if !matches.is_empty() {
                saw_new_data = true;
                write_matches(&mut stdout, output_format, query, &matches)?;
                stdout.flush()?;
            }
        }

        if !saw_new_data {
            thread::sleep(Duration::from_millis(200));
        }
    }
}

fn read_appended_matches(input: &mut FollowInput, query: &Query) -> Result<Vec<Match>, CliError> {
    let mut matches = Vec::new();

    input.reader.seek(SeekFrom::Start(input.position))?;

    loop {
        let mut line = String::new();
        let bytes_read = input.reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        input.position = input.reader.stream_position()?;
        trim_line_end(&mut line);

        let record = parse_line(&line);
        if let Some(matched) = execute_record(query, record) {
            matches.push(matched);
        }
    }

    Ok(matches)
}

fn trim_line_end(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
}

fn execute_stdin(stdin: impl BufRead, query: &Query) -> Result<Vec<Match>, CliError> {
    let mut matches = Vec::new();
    let limit = query.options.limit.unwrap_or(usize::MAX);

    for line in stdin.lines() {
        let record = parse_line(&line?);
        if let Some(matched) = execute_record(query, record) {
            matches.push(matched);

            if matches.len() >= limit {
                break;
            }
        }
    }

    Ok(matches)
}

fn write_matches(
    mut writer: impl Write,
    output_format: OutputFormat,
    query: &Query,
    matches: &[Match],
) -> Result<(), CliError> {
    for matched in matches {
        let line = format_match(matched, query, output_format).map_err(CliError::FormatRecord)?;
        writeln!(writer, "{line}")?;
    }

    Ok(())
}

fn write_explain(mut writer: impl Write, query: &Query) -> Result<(), CliError> {
    writeln!(writer, "{}", format_query_plan(query))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{Condition, Operator};
    use serde_json::json;

    #[test]
    fn run_module_exposes_entrypoint() {
        let run_fn: fn() -> Result<(), CliError> = run;

        assert_eq!(
            std::mem::size_of_val(&run_fn),
            std::mem::size_of::<fn() -> Result<(), CliError>>()
        );
    }

    #[test]
    fn executes_stdin_input() {
        let mut query = Query::empty();
        query.filters.must.push(Condition {
            field: "raw".to_string(),
            operator: Operator::Contains,
            value: Some(json!("ERROR")),
        });

        let matches = execute_stdin("INFO ok\nERROR failed\n".as_bytes(), &query).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].record.raw, "ERROR failed");
    }

    #[test]
    fn resolves_dsl_mode_positionals_as_inputs() {
        let (query, inputs) = resolve_query_and_inputs(
            Some(r#"{"version":1,"filters":{}}"#),
            &["app.log".to_string(), "other.log".to_string()],
        )
        .expect("query should resolve");

        assert!(query.filters.is_empty());
        assert_eq!(inputs, vec!["app.log", "other.log"]);
    }

    #[test]
    fn resolves_natural_language_mode_with_trailing_query() {
        let (query, inputs) =
            resolve_query_and_inputs(None, &["app.log".to_string(), "login failed".to_string()])
                .expect("query should resolve");

        assert_eq!(inputs, vec!["app.log"]);
        assert!(!query.filters.must.is_empty());
    }

    #[test]
    fn trims_line_endings_for_follow_lines() {
        let mut line = "ERROR failed\r\n".to_string();

        trim_line_end(&mut line);

        assert_eq!(line, "ERROR failed");
    }
}
