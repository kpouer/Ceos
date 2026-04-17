use egui::Color32;

#[derive(Debug, Clone)]
pub(crate) struct Highlight {
    pub(crate) text: String,
    pub(crate) case_insensitive: bool,
    pub(crate) color: Color32,
}

impl Highlight {
    pub(crate) const fn new(text: String, case_insensitive: bool, color: Color32) -> Self {
        Self {
            text,
            case_insensitive,
            color,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_highlight_rendering_multi_byte() {
        let text = "Hello, 🌍 world!";
        let pattern = "world";
        let search_text = text.to_string();

        let byte_index = search_text.find(pattern).unwrap();
        let char_index = search_text[..byte_index].chars().count();
        let char_count = pattern.chars().count();

        // H(0) e(1) l(2) l(3) o(4) ,(5)  (6) 🌍(7)  (8) w(9)
        // Correct is 9.
        assert_eq!(char_index, 9);
        assert_eq!(char_count, 5);
    }
}
