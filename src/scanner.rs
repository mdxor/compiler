pub trait Scanner<'source> {
  fn source(&mut self) -> &'source str;

  fn bytes(&mut self) -> &[u8];

  fn cur(&mut self) -> u8;

  fn forward(&mut self, size: usize);

  fn forward_slice(&mut self, size: usize) -> &'source str;

  fn scan_blank_line(&mut self) -> Option<usize> {
    let mut size = 0;
    for &b in self.bytes() {
      if b == b' ' {
        size += 1;
      } else if b == b'\n' {
        return Some(size);
      } else {
        return None;
      }
    }
    Some(size)
  }

  fn count_starting_whitespace(&mut self) -> usize {
    let mut size = 0;
    for &b in self.bytes() {
      if b == b' ' {
        size += 1;
      } else {
        break;
      }
    }
    size
  }

  fn skip_whitespace(&mut self) {
    let size = self.count_starting_whitespace();
    self.forward(size);
  }

  // return the keyword size, not the whole size
  fn scan_block_starting_token(&mut self, keyword: u8, max_size: usize) -> Option<usize> {
    let mut size = 0;
    for &b in self.bytes() {
      if b == b' ' || b == b'\n' {
        if size > 0 {
          self.forward(1);
        }
        break;
      } else if b == keyword {
        size += 1;
        if size > max_size {
          return None;
        }
      } else {
        return None;
      }
    }
    if size == 0 {
      None
    } else {
      self.forward(size);
      Some(size)
    }
  }

  fn match_keyword_cur_line(&mut self, keyword: u8, allow_internal_spaces: bool) -> bool {
    let mut size = 0;
    let mut starting_spaces = true;
    let mut ending_spaces = false;
    for &b in self.bytes() {
      if b == b'\n' {
        if size > 0 {
          size += 1;
        }
        break;
      }
      if starting_spaces {
        if b == keyword {
          starting_spaces = false;
          size += 1;
        } else if b == b' ' {
          size += 1
        } else {
          return false;
        }
      } else if ending_spaces {
        if b == b' ' {
          size += 1
        } else {
          return false;
        }
      } else {
        if b == keyword {
          size += 1
        } else if b == b' ' {
          if !allow_internal_spaces {
            ending_spaces = true;
          }
          size += 1;
        } else {
          return false;
        }
      }
    }
    if size == 0 {
      false
    } else {
      self.forward(size);
      true
    }
  }

  fn match_keywords_cur_line(&mut self, keywords: Vec<u8>, allow_internal_spaces: bool) -> bool {
    for keyword in keywords {
      if self.match_keyword_cur_line(keyword, allow_internal_spaces) {
        return true;
      }
    }
    return false;
  }
}
