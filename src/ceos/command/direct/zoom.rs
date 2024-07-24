use crate::ceos::textarea::textareaproperties::DEFAULT_LINE_HEIGHT;
use eframe::epaint::FontId;

#[derive(Debug, PartialEq)]
pub(crate) struct Zoom {
    size: f32,
}

impl Default for Zoom {
    fn default() -> Self {
        Self {
            size: DEFAULT_LINE_HEIGHT,
        }
    }
}

impl TryFrom<&str> for Zoom {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if command.trim() == "zoom reset" {
            return Ok(Default::default());
        }
        if let Some(stripped) = command.strip_prefix("zoom ") {
            if let Ok(size) = stripped.parse::<f32>() {
                return Ok(Zoom { size });
            }
        }
        Err(())
    }
}

impl Zoom {
    pub(crate) fn get_font_id(&self) -> FontId {
        FontId::new(self.size, egui::FontFamily::Monospace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(3.0, "zoom 3")]
    #[case(DEFAULT_LINE_HEIGHT, "zoom reset")]
    fn test_try_from(#[case] expected: f32, #[case] command: &str) -> anyhow::Result<(), ()> {
        let result = Zoom::try_from(command)?;
        assert_eq!(Zoom { size: expected }, result);
        Ok(())
    }

    #[rstest]
    #[case("zoo m 20")]
    #[case("zoom")]
    #[case("zoom a")]
    fn test_try_from_invalid(#[case] command: &str) -> anyhow::Result<(), ()> {
        assert!(Zoom::try_from(command).is_err());
        Ok(())
    }
}
