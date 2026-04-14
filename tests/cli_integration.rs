use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_loglens")
}

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be available")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("loglens-{name}-{suffix}"));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

fn write_file(path: &Path, contents: &str) {
    fs::write(path, contents).expect("file should be written");
}

#[test]
fn cli_filters_single_file_with_text_output() {
    let dir = temp_dir("single");
    let log = dir.join("app.log");
    write_file(&log, "INFO ready\nERROR failed\n");

    let output = Command::new(bin())
        .arg("--dsl")
        .arg(
            r#"{"version":1,"filters":{"must":[{"field":"raw","op":"contains","value":"ERROR"}]}}"#,
        )
        .arg(&log)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ERROR failed\n");
}

#[test]
fn cli_filters_multiple_files_and_applies_limit() {
    let dir = temp_dir("multiple");
    let first = dir.join("first.log");
    let second = dir.join("second.log");
    write_file(&first, "ERROR first\n");
    write_file(&second, "ERROR second\n");

    let output = Command::new(bin())
        .arg("--dsl")
        .arg(
            r#"{"version":1,"filters":{"must":[{"field":"raw","op":"contains","value":"ERROR"}]}}"#,
        )
        .arg("--limit")
        .arg("1")
        .arg(&first)
        .arg(&second)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ERROR first\n");
}

#[test]
fn cli_filters_glob_inputs() {
    let dir = temp_dir("glob");
    write_file(&dir.join("a.log"), "INFO ready\n");
    write_file(&dir.join("b.log"), "ERROR failed\n");

    let pattern = dir.join("*.log");
    let output = Command::new(bin())
        .arg("--dsl")
        .arg(
            r#"{"version":1,"filters":{"must":[{"field":"raw","op":"contains","value":"ERROR"}]}}"#,
        )
        .arg(pattern)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ERROR failed\n");
}

#[test]
fn cli_reads_dsl_from_file_and_outputs_json() {
    let dir = temp_dir("json");
    let log = dir.join("app.log");
    let dsl = dir.join("query.json");
    write_file(&log, r#"{"level":"ERROR","message":"login failed"}"#);
    write_file(
        &dsl,
        r#"{"version":1,"filters":{"must":[{"field":"message","op":"contains","value":"login"}]}}"#,
    );

    let output = Command::new(bin())
        .arg("--dsl")
        .arg(&dsl)
        .arg("--output")
        .arg("json")
        .arg(&log)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""raw":"{\"level\":\"ERROR\",\"message\":\"login failed\"}""#));
    assert!(stdout.contains(r#""message":"login failed""#));
}

#[test]
fn cli_reads_stdin_with_dash_input() {
    let mut child = Command::new(bin())
        .arg("--dsl")
        .arg(
            r#"{"version":1,"filters":{"must":[{"field":"raw","op":"contains","value":"ERROR"}]}}"#,
        )
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("command should spawn");

    child
        .stdin
        .as_mut()
        .expect("stdin should be open")
        .write_all(b"INFO ready\nERROR failed\n")
        .expect("stdin should be written");

    let output = child.wait_with_output().expect("command should finish");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ERROR failed\n");
}

#[test]
fn cli_filters_with_natural_language_query() {
    let dir = temp_dir("nl");
    let log = dir.join("app.log");
    write_file(
        &log,
        "INFO ready\nERROR login failed for user=alice\nERROR payment failed\n",
    );

    let output = Command::new(bin())
        .arg(&log)
        .arg("login failed")
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ERROR login failed for user=alice\n"
    );
}

#[test]
fn cli_explain_prints_validated_heuristic_dsl_to_stderr() {
    let dir = temp_dir("explain");
    let log = dir.join("app.log");
    write_file(&log, "INFO ready\nERROR login failed\n");

    let output = Command::new(bin())
        .arg("--explain")
        .arg(&log)
        .arg("login failed")
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ERROR login failed\n"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(r#""version": 1"#));
    assert!(stderr.contains(r#""value": "login failed""#));
}

#[test]
fn cli_follow_reads_appended_matching_lines() {
    let dir = temp_dir("follow");
    let log = dir.join("app.log");
    write_file(&log, "INFO existing line\n");

    let mut child = Command::new(bin())
        .arg("--follow")
        .arg("--dsl")
        .arg(
            r#"{"version":1,"filters":{"must":[{"field":"raw","op":"contains","value":"ERROR"}]}}"#,
        )
        .arg(&log)
        .stdout(Stdio::piped())
        .spawn()
        .expect("command should spawn");

    let stdout = child.stdout.take().expect("stdout should be open");
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let _ = reader.read_line(&mut line);
        let _ = tx.send(line);
    });

    let keep_appending = Arc::new(AtomicBool::new(true));
    let appender_keep_running = Arc::clone(&keep_appending);
    let appender_log = log.clone();
    let appender = thread::spawn(move || {
        while appender_keep_running.load(Ordering::SeqCst) {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&appender_log)
                .expect("log file should open");
            writeln!(file, "ERROR appended line").expect("line should append");
            thread::sleep(Duration::from_millis(100));
        }
    });

    let line = rx
        .recv_timeout(Duration::from_secs(3))
        .expect("follow output should arrive");
    keep_appending.store(false, Ordering::SeqCst);
    appender.join().expect("appender should finish");
    child.kill().expect("child should be killed");
    let _ = child.wait();

    assert_eq!(line, "ERROR appended line\n");
}
