// use super::document::*;
// use crate::byte::*;
// use crate::scan::*;
// use crate::token::*;
// use crate::tree::*;
// pub(crate) fn scan_table<'source>(
//   document: &Document<'source>,
//   tree: &mut Tree<Token<'source>>,
// ) -> Option<()> {
//   let start = document.offset();
//   let bytes = document.bytes();
//   let source = document.source();
//   let cur = tree.cur()?;
//   Some(())
// }

// // return size, cell count
// fn scan_table_row<'source>(bytes: &'source [u8], source: &'source str) -> (usize, usize) {
//   let raw_size = scan_raw_line_without_source(bytes);
//   let mut size = raw_size;
//   if bytes[raw_size - 1] == b'\n' {
//     size -= 1;
//     if bytes[raw_size - 2] == b'\r' {
//       size -= 1;
//     }
//   }
//   let mut col_start = 0;
//   let mut count = 0;
//   let mut starting = false;

//   let i = 0;
//   while i < size {
//     let c = bytes[i];
//     match c {
//       b'|' => {
//         if starting {
//           starting = false;
//           if i >= 1 {
//             if !source[..i - 1].trim().is_empty() {
//               count += 1;
//             }
//           }
//         } else {
//           count += 1;
//         }
//         col_start = i + 1;
//       }
//       b'\\' => {
//         if i + 1 < size {
//           i += 1;
//         }
//       }
//       _ => {}
//     }
//     i += 1;
//   }
//   if col_start < size - 1 {
//     if !source[col_start..size - 1].trim().is_empty() {
//       count += 1;
//     }
//   }
//   (raw_size, count)
// }

// #[derive(PartialEq)]
// enum AlignState {
//   Pending,
//   LeftColon,
//   Hyphen,
//   RightColon,
// }
// fn scan_table_aligns<'source>(bytes: &'source [u8]) -> Option<usize> {
//   let size = scan_raw_line_without_source(bytes);
//   let mut count = 0;

//   let i = 0;
//   let mut state = AlignState::Pending;
//   while i < size {
//     let c = bytes[i];
//     match c {
//       b'|' => if state == AlignState::RightColon || state == AlignState::Hyphen {},
//       b'-' => {}
//       b':' => {}
//       b'\n' | b'\r' => {}
//       _ => {
//         return None;
//       }
//     }
//     i += 1;
//   }
//   Some(count)
// }

// fn scan_table_row<'source>(bytes: &'source [u8]) {
//   let raw_size = scan_raw_line_without_source(bytes);
//   let mut size = raw_size;
//   if bytes[raw_size - 1] == b'\n' {
//     size -= 1;
//     if bytes[raw_size - 2] == b'\r' {
//       size -= 1;
//     }
//   }
//   let mut cols: Vec = vec![];
//   for c in &bytes[..size] {
//     match *c {
//       b' ' => {}
//       b'\\' => {}
//       b'|' => {}
//       _ => {}
//     }
//   }
// }

// fn scan_table_alignment<'source>(bytes: &'source [u8]) {}
