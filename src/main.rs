fn main() {
    if let Err(error) = loglens::cli::run() {
        eprintln!("loglens: {error}");
        std::process::exit(1);
    }
}
