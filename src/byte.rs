use crate::punctuation::*;
pub(crate) fn is_ascii_punctuation(c: u8) -> bool {
  c < 128 && (PUNCT_MASKS_ASCII[(c / 16) as usize] & (1 << (c & 15))) != 0
}

pub(crate) fn is_punctuation(c: char) -> bool {
  let cp = c as u32;
  if cp < 128 {
    return is_ascii_punctuation(cp as u8);
  }
  if cp > 0x1BC9F {
    return false;
  }
  let high = (cp / 16) as u16;
  match PUNCT_TAB.binary_search(&high) {
    Ok(index) => (PUNCT_MASKS[index] & (1 << (cp & 15))) != 0,
    _ => false,
  }
}

pub(crate) fn is_ascii_whitespace(c: u8) -> bool {
  (c >= 0x09 && c <= 0x0d) || c == b' '
}
pub(crate) fn is_ascii_whitespace_no_nl(c: u8) -> bool {
  c == b'\t' || c == 0x0b || c == 0x0c || c == b' '
}
pub(crate) fn is_valid_unquoted_attr_value_char(c: u8) -> bool {
  !matches!(
    c,
    b'\'' | b'"' | b' ' | b'=' | b'>' | b'<' | b'`' | b'\n' | b'\r'
  )
}
