use crate::ceos::gui::theme::Theme;
use eframe::epaint::Color32;

static DIGIT: Color32 = Color32::from_rgb(0xff, 0, 0);
#[allow(dead_code)]
static KEYWORD1: Color32 = Color32::from_rgb(0x00, 0x66, 0x99);
#[allow(dead_code)]
static KEYWORD2: Color32 = Color32::from_rgb(0x00, 0x99, 0x66);
#[allow(dead_code)]
static KEYWORD3: Color32 = Color32::from_rgb(0x00, 0x99, 0xff);
#[allow(dead_code)]
static KEYWORD4: Color32 = Color32::from_rgb(0x66, 0xcc, 0xff);
static LITERAL1: Color32 = Color32::from_rgb(0xff, 0x00, 0xcc);
static LITERAL2: Color32 = Color32::from_rgb(0xcc, 0x00, 0xcc);
#[allow(dead_code)]
static LITERAL3: Color32 = Color32::from_rgb(0x99, 0x00, 0xcc);
#[allow(dead_code)]
static LITERAL4: Color32 = Color32::from_rgb(0x66, 0x00, 0xcc);
#[allow(dead_code)]
static MARKUP: Color32 = Color32::from_rgb(0x00, 0x00, 0xff);
static OPERATOR: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);

impl Theme {
    #[allow(non_snake_case)]
    pub(crate) const fn jEdit() -> Self {
        Self {
            dark: false,
            background: Color32::WHITE,
            background_faint: Color32::from_rgb(0xdb, 0xdb, 0xdb),
            text: Color32::BLACK,
            text_faint: Color32::BLACK,
            info: Color32::BLUE,
            warning: Color32::YELLOW,
            error: Color32::RED,
            literal: LITERAL2,
            operator: OPERATOR,
            number: DIGIT,
            string: LITERAL1,
            deleting: Color32::RED,
        }
    }
}
