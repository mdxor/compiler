use crate::scan::{is_ascii_whitespace_no_nl, scan_blank_line, scan_hrule};
#[derive(Clone)]
pub(crate) struct LineStart<'a> {
  bytes: &'a [u8],
  tab_start: usize,
  ix: usize,
  spaces_remaining: usize,
  min_hrule_offset: usize,
}

impl<'a> LineStart<'a> {
  pub(crate) fn new(bytes: &[u8]) -> LineStart {
    LineStart {
      bytes,
      tab_start: 0,
      ix: 0,
      spaces_remaining: 0,
      min_hrule_offset: 0,
    }
  }

  /// Try to scan a number of spaces.
  ///
  /// Returns true if all spaces were consumed.
  ///
  /// Note: consumes some spaces even if not successful.
  pub(crate) fn scan_space(&mut self, n_space: usize) -> bool {
    self.scan_space_inner(n_space) == 0
  }

  /// Scan a number of spaces up to a maximum.
  ///
  /// Returns number of spaces scanned.
  pub(crate) fn scan_space_upto(&mut self, n_space: usize) -> usize {
    n_space - self.scan_space_inner(n_space)
  }

  /// Returns unused remainder of spaces.
  fn scan_space_inner(&mut self, mut n_space: usize) -> usize {
    let n_from_remaining = self.spaces_remaining.min(n_space);
    self.spaces_remaining -= n_from_remaining;
    n_space -= n_from_remaining;
    while n_space > 0 && self.ix < self.bytes.len() {
      match self.bytes[self.ix] {
        b' ' => {
          self.ix += 1;
          n_space -= 1;
        }
        b'\t' => {
          let spaces = 4 - (self.ix - self.tab_start) % 4;
          self.ix += 1;
          self.tab_start = self.ix;
          let n = spaces.min(n_space);
          n_space -= n;
          self.spaces_remaining = spaces - n;
        }
        _ => break,
      }
    }
    n_space
  }

  /// Scan all available ASCII whitespace (not including eol).
  pub(crate) fn scan_all_space(&mut self) {
    self.spaces_remaining = 0;
    self.ix += self.bytes[self.ix..]
      .iter()
      .take_while(|&&b| b == b' ' || b == b'\t')
      .count();
  }

  /// Determine whether we're at end of line (includes end of file).
  pub(crate) fn is_at_eol(&self) -> bool {
    self
      .bytes
      .get(self.ix)
      .map(|&c| c == b'\r' || c == b'\n')
      .unwrap_or(true)
  }

  fn scan_ch(&mut self, c: u8) -> bool {
    if self.ix < self.bytes.len() && self.bytes[self.ix] == c {
      self.ix += 1;
      true
    } else {
      false
    }
  }

  pub(crate) fn scan_blockquote_marker(&mut self) -> bool {
    let save = self.clone();
    let _ = self.scan_space(3);
    if self.scan_ch(b'>') {
      let _ = self.scan_space(1);
      true
    } else {
      *self = save;
      false
    }
  }

  /// Scan a list marker.
  ///
  /// Return value is the character, the start index, and the indent in spaces.
  /// For ordered list markers, the character will be one of b'.' or b')'. For
  /// bullet list markers, it will be one of b'-', b'+', or b'*'.
  pub(crate) fn scan_list_marker(&mut self) -> Option<(u8, u64, usize)> {
    let save = self.clone();
    let indent = self.scan_space_upto(4);
    if indent < 4 && self.ix < self.bytes.len() {
      let c = self.bytes[self.ix];
      if c == b'-' || c == b'+' || c == b'*' {
        if self.ix >= self.min_hrule_offset {
          // there could be an hrule here
          if let Err(min_offset) = scan_hrule(&self.bytes[self.ix..]) {
            self.min_hrule_offset = min_offset;
          } else {
            *self = save;
            return None;
          }
        }
        self.ix += 1;
        if self.scan_space(1) || self.is_at_eol() {
          return self.finish_list_marker(c, 0, indent + 2);
        }
      } else if c >= b'0' && c <= b'9' {
        let start_ix = self.ix;
        let mut ix = self.ix + 1;
        let mut val = u64::from(c - b'0');
        while ix < self.bytes.len() && ix - start_ix < 10 {
          let c = self.bytes[ix];
          ix += 1;
          if c >= b'0' && c <= b'9' {
            val = val * 10 + u64::from(c - b'0');
          } else if c == b')' || c == b'.' {
            self.ix = ix;
            if self.scan_space(1) || self.is_at_eol() {
              return self.finish_list_marker(c, val, indent + self.ix - start_ix);
            } else {
              break;
            }
          } else {
            break;
          }
        }
      }
    }
    *self = save;
    None
  }

  fn finish_list_marker(
    &mut self,
    c: u8,
    start: u64,
    mut indent: usize,
  ) -> Option<(u8, u64, usize)> {
    let save = self.clone();

    // skip the rest of the line if it's blank
    if scan_blank_line(&self.bytes[self.ix..]).is_some() {
      return Some((c, start, indent));
    }

    let post_indent = self.scan_space_upto(4);
    if post_indent < 4 {
      indent += post_indent;
    } else {
      *self = save;
    }
    Some((c, start, indent))
  }

  /// Returns Some(is_checked) when a task list marker was found. Resets itself
  /// to original state otherwise.
  pub(crate) fn scan_task_list_marker(&mut self) -> Option<bool> {
    let save = self.clone();
    self.scan_space_upto(3);

    if !self.scan_ch(b'[') {
      *self = save;
      return None;
    }
    let is_checked = match self.bytes.get(self.ix) {
      Some(&c) if is_ascii_whitespace_no_nl(c) => {
        self.ix += 1;
        false
      }
      Some(b'x') | Some(b'X') => {
        self.ix += 1;
        true
      }
      _ => {
        *self = save;
        return None;
      }
    };
    if !self.scan_ch(b']') {
      *self = save;
      return None;
    }
    if !self
      .bytes
      .get(self.ix)
      .map(|&b| is_ascii_whitespace_no_nl(b))
      .unwrap_or(false)
    {
      *self = save;
      return None;
    }
    Some(is_checked)
  }

  pub(crate) fn bytes_scanned(&self) -> usize {
    self.ix
  }

  pub(crate) fn remaining_space(&self) -> usize {
    self.spaces_remaining
  }
}
