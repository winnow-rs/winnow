use crate::combinator::dispatch;
use crate::combinator::fail;
use crate::combinator::success;
use crate::error::ErrMode;
use crate::error::ErrorKind;
use crate::error::ParserError;
use crate::prelude::*;
use crate::token::any;

#[test]
fn basics() {
    fn escape_seq_char(input: &mut &str) -> PResult<char> {
        dispatch! {any;
            'b' => success('\u{8}'),
            'f' => success('\u{c}'),
            'n' => success('\n'),
            'r' => success('\r'),
            't' => success('\t'),
            '\\' => success('\\'),
            '"' => success('"'),
            _ => fail::<_, char, _>,
        }
        .parse_next(input)
    }
    assert_eq!(escape_seq_char.parse_peek("b123"), Ok(("123", '\u{8}')));
    assert_eq!(
        escape_seq_char.parse_peek("error"),
        Err(ErrMode::Backtrack(ParserError::from_error_kind(
            &"rror",
            ErrorKind::Fail
        )))
    );
    assert_eq!(
        escape_seq_char.parse_peek(""),
        Err(ErrMode::Backtrack(ParserError::from_error_kind(
            &"",
            ErrorKind::Fail
        )))
    );
}
