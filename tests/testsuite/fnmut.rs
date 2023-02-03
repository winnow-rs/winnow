#![cfg(feature = "alloc")]

use winnow::bytes::tag;
#[cfg(feature = "std")]
use winnow::multi::many0;
use winnow::multi::many0_count;

#[test]
#[cfg(feature = "std")]
fn parse() {
  let mut counter = 0;

  let res = {
    let mut parser = many0::<_, _, Vec<_>, (), _>(|i| {
      counter += 1;
      tag("abc")(i)
    });

    parser("abcabcabcabc").unwrap()
  };

  println!("res: {:?}", res);
  assert_eq!(counter, 5);
}

#[test]
fn accumulate() {
  let mut v = Vec::new();

  let (_, count) = {
    let mut parser = many0_count::<_, _, (), _>(|i| {
      let (i, o) = tag("abc")(i)?;
      v.push(o);
      Ok((i, ()))
    });
    parser("abcabcabcabc").unwrap()
  };

  println!("v: {:?}", v);
  assert_eq!(count, 4);
  assert_eq!(v.len(), 4);
}
