#[derive(Debug)]
pub(crate) struct Selection {
    pub(crate) start_column: usize,
    pub(crate) end_column: usize,
    pub(crate) line: usize,
}
