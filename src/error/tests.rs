use super::*;

mod longest_match {
    use super::*;
    use crate::combinator::{alt, eof};

    type Error<'a> = LongestMatch<&'a str, ContextError<&'static str>>;

    fn pattern<'a, O>(
        p: impl Parser<&'a str, O, Error<'a>>,
        label: &'static str,
    ) -> impl Parser<&'a str, &'a str, Error<'a>> {
        (p, eof).recognize().context(label)
    }

    #[test]
    fn parser_error_implementation() {
        let input = "abcd";
        let checkpoint1 = &&input[2..];
        let checkpoint2 = &&input[3..];

        assert_eq!(
            Error::new()
                .append(checkpoint1, ErrorKind::Token)
                .append(checkpoint2, ErrorKind::Tag),
            Error::from_error_kind(checkpoint2, ErrorKind::Tag),
        );

        assert_eq!(
            Error::new()
                .append(checkpoint2, ErrorKind::Tag)
                .append(checkpoint1, ErrorKind::Token),
            Error::from_error_kind(checkpoint2, ErrorKind::Tag),
        );

        assert_eq!(
            Error::new()
                .append(checkpoint1, ErrorKind::Token)
                .append(checkpoint1, ErrorKind::Tag),
            Error::from_error_kind(checkpoint1, ErrorKind::Token),
        );

        assert_eq!(
            Error::from_error_kind(checkpoint1, ErrorKind::Token)
                .or(Error::from_error_kind(checkpoint2, ErrorKind::Tag)),
            Error::from_error_kind(checkpoint2, ErrorKind::Tag),
        );

        assert_eq!(
            Error::from_error_kind(checkpoint2, ErrorKind::Tag)
                .or(Error::from_error_kind(checkpoint1, ErrorKind::Token)),
            Error::from_error_kind(checkpoint2, ErrorKind::Tag),
        );

        assert_eq!(
            Error::from_error_kind(checkpoint1, ErrorKind::Token)
                .or(Error::from_error_kind(checkpoint1, ErrorKind::Tag)),
            Error::from_error_kind(checkpoint1, ErrorKind::Token),
        );
    }

    #[test]
    fn add_context() {
        let input = "abcd";
        let checkpoint1 = &&input[2..];
        let checkpoint2 = &&input[3..];

        assert_eq!(
            Error::new()
                .add_context(checkpoint1, "don't want")
                .add_context(checkpoint2, "want"),
            Error::new().add_context(checkpoint2, "want"),
        );

        assert_eq!(
            Error::new()
                .add_context(checkpoint2, "want")
                .add_context(checkpoint1, "don't want"),
            Error::new().add_context(checkpoint2, "want"),
        );

        assert_eq!(
            Error::new()
                .add_context(checkpoint1, "want")
                .add_context(checkpoint1, "also want")
                .into_inner(),
            ContextError::new()
                .add_context(checkpoint1, "want")
                .add_context(checkpoint1, "also want"),
        );
    }

    #[test]
    fn merge_context() {
        let input = "abcd";
        let checkpoint1 = &&input[2..];
        let checkpoint2 = &&input[3..];

        assert_eq!(
            Error::new()
                .add_context(checkpoint1, "don't want")
                .clear_context(),
            Error::new(),
        );

        assert_eq!(
            Error::new()
                .add_context(checkpoint1, "don't want")
                .merge_context(Error::new().add_context(checkpoint2, "want")),
            Error::new().add_context(checkpoint2, "want"),
        );

        assert_eq!(
            Error::new()
                .add_context(checkpoint2, "want")
                .merge_context(Error::new().add_context(checkpoint1, "don't want")),
            Error::new().add_context(checkpoint2, "want"),
        );

        assert_eq!(
            Error::new()
                .add_context(checkpoint1, "want")
                .merge_context(Error::new().add_context(checkpoint1, "also want"))
                .into_inner(),
            ContextError::new()
                .add_context(checkpoint1, "want")
                .add_context(checkpoint1, "also want"),
        );
    }

    #[test]
    fn single_longest_match_first_in_alt() {
        let mut parser = alt((
            pattern(('a', 'b', 'c', 'd'), "wanted"),
            pattern(('a', 'b', 'c'), "don't want 1"),
            pattern(('a', 'b'), "don't want 2"),
        ));

        let mut input = "abcde";
        let checkpoint = &&input[4..]; // 4 characters consumed by longest match
        assert_eq!(
            parser.parse_next(&mut input),
            Err(ErrMode::Backtrack(
                LongestMatch::new().add_context(checkpoint, "wanted"),
            ))
        );
    }

    #[test]
    fn multi_longest_match() {
        let mut parser = alt((
            pattern(('d', 'e', 'f'), "don't want"),
            pattern(('a', 'b', 'c', 'd'), "wanted 1"),
            pattern(('a', 'b', 'c'), "wanted 2"),
            pattern(('d', 'e', 'f', 'g'), "don't want"),
        ));

        let mut input = "abd";
        let checkpoint = &&input[2..]; // 2 characters consumed by longest match
        assert_eq!(
            parser.parse_next(&mut input),
            Err(ErrMode::Backtrack(
                LongestMatch::new()
                    .add_context(checkpoint, "wanted 1")
                    .add_context(checkpoint, "wanted 2"),
            ))
        );
    }

    #[test]
    fn multi_longest_match_input_short() {
        let mut parser = alt((
            pattern(('d', 'e', 'f'), "don't want"),
            pattern(('a', 'b', 'c', 'd'), "wanted 1"),
            pattern(('a', 'b', 'c'), "wanted 2"),
            pattern(('d', 'e', 'f', 'g'), "don't want"),
        ));

        let mut input = "ab";
        let checkpoint = &&input[2..]; // 2 characters consumed by longest match
        assert_eq!(
            parser.parse_next(&mut input),
            Err(ErrMode::Backtrack(
                LongestMatch::new()
                    .add_context(checkpoint, "wanted 1")
                    .add_context(checkpoint, "wanted 2"),
            ))
        );
    }
}
