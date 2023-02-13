use super::*;

#[test]
fn test_offset_u8() {
    let s = b"abcd123";
    let a = &s[..];
    let b = &a[2..];
    let c = &a[..4];
    let d = &a[3..5];
    assert_eq!(a.offset_to(b), 2);
    assert_eq!(a.offset_to(c), 0);
    assert_eq!(a.offset_to(d), 3);
}

#[test]
fn test_offset_str() {
    let a = "abcřèÂßÇd123";
    let b = &a[7..];
    let c = &a[..5];
    let d = &a[5..9];
    assert_eq!(a.offset_to(b), 7);
    assert_eq!(a.offset_to(c), 0);
    assert_eq!(a.offset_to(d), 5);
}
