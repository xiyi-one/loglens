#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub raw_logs_stay_local: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            raw_logs_stay_local: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_local_logs() {
        let config = Config::default();

        assert!(config.raw_logs_stay_local);
    }
}
