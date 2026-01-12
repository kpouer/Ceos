use crate::ceos::gui::theme::Theme;

impl Theme {
    pub(crate) const fn solarized_dark() -> Theme {
        Theme {
            dark: true,
            background: egui_solarized::BASE03,
            background_faint: egui_solarized::BASE02,
            text: egui_solarized::BASE0,
            text_faint: egui_solarized::BASE1,
            info: egui_solarized::BLUE,
            warning: egui_solarized::ORANGE,
            error: egui_solarized::RED,
            literal: egui_solarized::BASE0,
            operator: egui_solarized::BLUE,
            number: egui_solarized::YELLOW,
            string: egui_solarized::GREEN,
            deleting: egui_solarized::RED,
        }
    }

    pub(crate) const fn solarized_light() -> Theme {
        Theme {
            dark: false,
            background: egui_solarized::BASE3,
            background_faint: egui_solarized::BASE2,
            ..Self::solarized_dark()
        }
    }
}
