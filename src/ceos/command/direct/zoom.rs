use eframe::epaint::FontId;

pub(crate) struct Zoom {
    size: f32,
}

impl TryFrom<&str> for Zoom {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if let Some(stripped) = command.strip_prefix("zoom ") {
            if let Ok(size) = stripped.parse::<f32>() {
                Ok(Zoom { size })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl Zoom {
    pub(crate) fn get_font_id(&self) -> FontId {
        FontId::new(self.size, egui::FontFamily::Monospace)
    }
}
