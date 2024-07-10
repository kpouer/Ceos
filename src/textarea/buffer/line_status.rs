#[derive(Default, PartialEq, Debug)]
pub(crate) enum LineStatus {
    #[default]
    Normal,
    Unmodified,
    Dirty,
}
