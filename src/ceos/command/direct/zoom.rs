use eframe::epaint::FontId;

#[derive(Debug, PartialEq)]
pub(crate) struct Zoom {
    size: f32,
}

impl TryFrom<&str> for Zoom {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if let Some(stripped) = command.strip_prefix("zoom ") {
            if let Ok(size) = stripped.parse::<f32>() {
                return Ok(Zoom { size });
            }
        }
        return Err(());
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

    #[test]
    fn test_try_from() -> anyhow::Result<(), ()> {
        let result = Zoom::try_from("zoom 20")?;
        assert_eq!(Zoom { size: 20.0 }, result);
        Ok(())
    }

    #[test]
    fn test_try_from_invalid() -> anyhow::Result<(), ()> {
        assert!(Zoom::try_from("zoo m 20").is_err());
        Ok(())
    }

    #[test]
    fn test_try_from_invalid2() -> anyhow::Result<(), ()> {
        assert!(Zoom::try_from("zoom").is_err());
        Ok(())
    }

    #[test]
    fn test_try_from_invalid3() -> anyhow::Result<(), ()> {
        assert!(Zoom::try_from("zoom a").is_err());
        Ok(())
    }
}
