/// Sequences multiple parsers and builds a struct out of them.
///
///# Example
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::ascii::{alphanumeric1, dec_uint, space0};
/// # use winnow::combinator::delimited;
/// # use winnow::error::ContextError;
/// use winnow::combinator::seq;
///
/// #[derive(Debug, PartialEq)]
/// struct Point(u32, u32);
///
/// #[derive(Debug, PartialEq)]
/// struct Field {
///     name: Vec<u8>,
///     value: Vec<u8>,
///     point: Point,
/// }
///
/// let num = dec_uint::<_, u32, ContextError>;
/// let spaced = |b| delimited(space0, b, space0);
/// let mut parser = seq!{
///     Field {
///         name: alphanumeric1.map(|s: &[u8]| s.to_owned()),
///         // `_` fields are ignored when building the struct
///         _: spaced(b':'),
///         value: alphanumeric1.map(|s: &[u8]| s.to_owned()),
///         _: spaced(b':'),
///         point: (num, spaced(b','), num).map(|(x, _, y)| Point(x, y),)
///     }
/// };
/// assert_eq!(
///     parser.parse_peek(&b"test: data: 123 , 4"[..]),
///     Ok((
///         &b""[..],
///         Field {
///             name: b"test"[..].to_owned(),
///             value: b"data"[..].to_owned(),
///             point: Point(123, 4),
///         },
///     )),
/// );
/// ```
#[macro_export]
#[doc(alias = "tuple")]
#[doc(alias = "preceded")]
#[doc(alias = "terminated")]
#[doc(alias = "delimited")]
#[doc(alias = "pair")]
#[doc(alias = "separated_pair")]
#[doc(alias = "struct_parser")]
macro_rules! seq {
    ($name: ident { $($fields: tt)* }) => {
        $crate::trace::trace(stringify!($name), move |input: &mut _| {
            use $crate::Parser;
            $crate::seq_parse_struct_fields!(input; $($fields)*);
            #[allow(clippy::redundant_field_names)]
            Ok($crate::seq_init_struct_fields!( ($($fields)*); $name;))
        })
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! seq_parse_struct_fields {
    (
        $input: ident;
        _ : $head_parser: expr, $($fields: tt)*
    ) => {
        let _ = $head_parser.parse_next($input)?;
        $crate::seq_parse_struct_fields!($input; $($fields)*)
    };
    (
        $input: ident;
        _ : $head_parser: expr
    ) => {
        let _ = $head_parser.parse_next($input)?;
    };
    (
        $input: ident;
        $head_field: ident : $head_parser: expr, $($fields: tt)*
    ) => {
        let $head_field = $head_parser.parse_next($input)?;
        $crate::seq_parse_struct_fields!($input; $($fields)*)
    };
    (
        $input: ident;
        $head_field: ident : $head_parser: expr
    ) => {
        let $head_field = $head_parser.parse_next($input)?;
    };
    (
        $input: expr;
        $(,)?
    ) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! seq_init_struct_fields {
    (
        (_ : $head_parser: expr, $($fields: tt)*);
        $name: ident;
        $($inits: tt)*
    ) => {
        $crate::seq_init_struct_fields!( ( $($fields)* ) ; $name ; $($inits)* )
    };
    (
        (_ : $head_parser: expr);
        $name: ident;
        $($inits: tt)*
    ) => {
        $crate::seq_init_struct_fields!( (); $name ; $($inits)* )
    };
    (
        ($head_field: ident : $head_parser: expr, $($fields: tt)*);
        $name: ident;
        $($inits: tt)*
    ) =>
    {
        $crate::seq_init_struct_fields!( ( $($fields)* ) ; $name ; $($inits)* $head_field: $head_field, )
    };
    (
        ($head_field: ident : $head_parser: expr);
        $name: ident;
        $($inits: tt)*
    ) => {
        $crate::seq_init_struct_fields!( (); $name ; $($inits)* $head_field: $head_field,)
    };
    (
        ($(,)?);
        $name: ident;
        $($inits: tt)*
    ) => {
        $name { $($inits)* }
    };
}
