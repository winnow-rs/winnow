//! Parser execution tracing

#[cfg(feature = "debug")]
mod internals;

use crate::error::ErrMode;
use crate::stream::Stream;
use crate::IResult;
use crate::Parser;

#[cfg(all(feature = "debug", not(feature = "std")))]
compile_error!("`debug` requires `std`");

/// Trace the execution of the parser
#[cfg_attr(not(feature = "debug"), allow(unused_variables))]
pub fn trace<I: Stream, O, E>(
    name: impl crate::lib::std::fmt::Display,
    mut parser: impl Parser<I, O, E>,
) -> impl FnMut(I) -> IResult<I, O, E> {
    #[cfg(feature = "debug")]
    {
        let mut call_count = 0;
        move |i| {
            let depth = internals::Depth::new();
            let original = i.clone();
            internals::start(*depth, &name, call_count, &original);

            let res = parser.parse_next(i);

            let consumed = res.as_ref().ok().map(|(i, _)| original.offset_to(i));
            let severity = internals::Severity::with_result(&res);
            internals::end(*depth, &name, call_count, consumed, severity);
            call_count += 1;

            res
        }
    }
    #[cfg(not(feature = "debug"))]
    {
        move |i| parser.parse_next(i)
    }
}

#[cfg_attr(not(feature = "debug"), allow(unused_variables))]
pub(crate) fn trace_result<T, E>(
    name: impl crate::lib::std::fmt::Display,
    res: &Result<T, ErrMode<E>>,
) {
    #[cfg(feature = "debug")]
    {
        let depth = internals::Depth::existing();
        let severity = internals::Severity::with_result(res);
        internals::result(*depth, &name, severity);
    }
}
