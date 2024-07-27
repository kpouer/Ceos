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

    pub(crate) fn contains(&self, value: usize) -> bool {
        if value < self.start {
            return false;
        }
        match self.end {
            Some(end) => value < end,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(3, Some(22), "3..22")]
    #[case(0, Some(22), "..22")]
    #[case(3, None, "3..")]
    fn test_try_from(
        #[case] start: usize,
        #[case] end: Option<usize>,
        #[case] command: &str,
    ) -> anyhow::Result<(), ()> {
        let result = Range::try_from(command)?;
        assert_eq!(Range { start, end }, result);
        Ok(())
    }

    #[rstest]
    #[case("33")]
    #[case("33..22")]
    #[case("33.2")]
    #[case("..")]
    #[case("..-22")]
    #[case("-3..")]
    #[case("-3..-4")]
    fn test_try_from_invalid(#[case] command: &str) -> anyhow::Result<(), ()> {
        assert!(Range::try_from(command).is_err());
        Ok(())
    }

    #[rstest]
    #[case(0, "3..10", false)]
    #[case(3, "3..10", true)]
    #[case(5, "3..10", true)]
    #[case(10, "3..10", false)]
    #[case(12, "3..10", false)]
    #[case(12, "3..", true)]
    fn test_contains(
        #[case] value: usize,
        #[case] command: &str,
        #[case] expected: bool,
    ) -> Result<(), ()> {
        let result = Range::try_from(command)?;
        assert_eq!(expected, result.contains(value));
        Ok(())
    }
}
