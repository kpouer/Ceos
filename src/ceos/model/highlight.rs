#[derive(Default)]
pub(crate) struct Highlight {
    pattern: String,
}

impl From<&str> for Highlight {
    fn from(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
        }
    }
}

impl AsRef<str> for Highlight {
    fn as_ref(&self) -> &str {
        &self.pattern
    }
}

impl Highlight {
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}
