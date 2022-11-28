use super::*;
use crate::error::Error;
use crate::sequence::tuple;

#[test]
/// Take the `bits` function and assert that remaining bytes are correctly returned, if the
/// previous bytes are fully consumed
fn test_complete_byte_consumption_bits() {
  use crate::bits::streaming::take;

  let input = &[0x12, 0x34, 0x56, 0x78];

  // Take 3 bit slices with sizes [4, 8, 4].
  let result: IResult<&[u8], (u8, u8, u8)> =
    bits::<_, _, Error<(&[u8], usize)>, _, _>(tuple((take(4usize), take(8usize), take(4usize))))(
      input,
    );

  let output = result.expect("We take 2 bytes and the input is longer than 2 bytes");

  let remaining = output.0;
  assert_eq!(remaining, [0x56, 0x78]);

  let parsed = output.1;
  assert_eq!(parsed.0, 0x01);
  assert_eq!(parsed.1, 0x23);
  assert_eq!(parsed.2, 0x04);
}

#[test]
/// Take the `bits` function and assert that remaining bytes are correctly returned, if the
/// previous bytes are NOT fully consumed. Partially consumed bytes are supposed to be dropped.
/// I.e. if we consume 1.5 bytes of 4 bytes, 2 bytes will be returned, bits 13-16 will be
/// dropped.
fn test_partial_byte_consumption_bits() {
  use crate::bits::streaming::take;

  let input = &[0x12, 0x34, 0x56, 0x78];

  // Take bit slices with sizes [4, 8].
  let result: IResult<&[u8], (u8, u8)> =
    bits::<_, _, Error<(&[u8], usize)>, _, _>(tuple((take(4usize), take(8usize))))(input);

  let output = result.expect("We take 1.5 bytes and the input is longer than 2 bytes");

  let remaining = output.0;
  assert_eq!(remaining, [0x56, 0x78]);

  let parsed = output.1;
  assert_eq!(parsed.0, 0x01);
  assert_eq!(parsed.1, 0x23);
}

#[test]
#[cfg(feature = "std")]
/// Ensure that in Incomplete error is thrown, if too few bytes are passed for a given parser.
fn test_incomplete_bits() {
  use crate::bits::streaming::take;
  let input = &[0x12];

  // Take bit slices with sizes [4, 8].
  let result: IResult<&[u8], (u8, u8)> =
    bits::<_, _, Error<(&[u8], usize)>, _, _>(tuple((take(4usize), take(8usize))))(input);

  assert!(result.is_err());
  let error = result.err().unwrap();
  assert_eq!("Parsing requires 2 bytes/chars", error.to_string());
}
