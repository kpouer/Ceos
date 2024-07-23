const SEPARATOR: &str = "..";

#[derive(Debug, PartialEq)]
pub(crate) struct Range {
    pub(crate) start: usize,
    pub(crate) end: Option<usize>,
}

impl TryFrom<&str> for Range {
    type Error = ();

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        if let Some(stripped) = command.strip_prefix(SEPARATOR) {
            let end = stripped.parse::<usize>().ok();
            if end.is_some() {
                return Ok(Range { start: 0, end });
            }
        } else if let Some(stripped) = command.strip_suffix(SEPARATOR) {
            if let Ok(start) = stripped.parse::<usize>() {
                return Ok(Range { start, end: None });
            }
        } else {
            let tokens: Vec<&str> = command.split(SEPARATOR).collect();
            if tokens.len() == 2 {
                if let Ok(start) = tokens.first().unwrap().parse::<usize>() {
                    if let Ok(end) = tokens.get(1).unwrap().parse::<usize>() {
                        return Range::new(start, end);
                    }
                }
            }
        }
        Err(())
    }
}

impl Range {
    fn new(start: usize, end: usize) -> Result<Range, ()> {
        if start > end {
            return Err(());
        }
        Ok(Range {
            start,
            end: Some(end),
        })
    }
}
