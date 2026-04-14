#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    pub name: String,
    pub query: String,
}

impl Alias {
    pub fn new(name: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            query: query.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_alias() {
        let alias = Alias::new("errors", "show errors");

        assert_eq!(alias.name, "errors");
        assert_eq!(alias.query, "show errors");
    }
}
