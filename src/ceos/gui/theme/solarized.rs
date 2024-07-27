use egui::Color32;

use crate::ceos::gui::theme::Theme;

static BASE03: Color32 = Color32::from_rgb(0x00, 0x2b, 0x36);
static BASE02: Color32 = Color32::from_rgb(0x07, 0x36, 0x42);
#[allow(dead_code)]
static BASE01: Color32 = Color32::from_rgb(0x58, 0x6e, 0x75);
#[allow(dead_code)]
static BASE00: Color32 = Color32::from_rgb(0x65, 0x7b, 0x83);
static BASE0: Color32 = Color32::from_rgb(0x83, 0x94, 0x96);
static BASE1: Color32 = Color32::from_rgb(0x93, 0xa1, 0xa1);
static BASE2: Color32 = Color32::from_rgb(0xee, 0xe8, 0xd5);
static BASE3: Color32 = Color32::from_rgb(0xfd, 0xf6, 0xe3);
static YELLOW: Color32 = Color32::from_rgb(0xb5, 0x89, 0x00);
static ORANGE: Color32 = Color32::from_rgb(0xcb, 0x4b, 0x16);
static RED: Color32 = Color32::from_rgb(0xdc, 0x32, 0x2f);
#[allow(dead_code)]
static MAGENTA: Color32 = Color32::from_rgb(0xd3, 0x36, 0x82);
#[allow(dead_code)]
static VIOLET: Color32 = Color32::from_rgb(0x6c, 0x71, 0xc4);
static BLUE: Color32 = Color32::from_rgb(0x26, 0x8b, 0xd2);
#[allow(dead_code)]
static CYAN: Color32 = Color32::from_rgb(0x2a, 0xa1, 0x98);
static GREEN: Color32 = Color32::from_rgb(0x85, 0x99, 0x00);

impl Theme {
    pub(crate) fn solarized_dark() -> Theme {
        Theme {
            dark: true,
            background: BASE03,
            background_faint: BASE02,
            text: BASE0,
            text_faint: BASE1,
            info: BLUE,
            warning: ORANGE,
            error: RED,
            literal: BASE0,
            operator: BLUE,
            number: YELLOW,
            string: GREEN,
            deleting: RED,
        }
    }

    pub(crate) fn solarized_light() -> Theme {
        Theme {
            dark: true,
            background: BASE3,
            background_faint: BASE2,
            text: BASE0,
            text_faint: BASE1,
            info: BLUE,
            warning: ORANGE,
            error: RED,
            literal: BASE0,
            operator: BLUE,
            number: YELLOW,
            string: GREEN,
            deleting: RED,
        }
    }
}
