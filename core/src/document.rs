use crate::lexer::*;
use crate::token::*;
pub struct Document<'source> {
  pub bytes: &'source [u8],
  spaces: usize,
  start: usize,
}

impl<'source> Document<'source> {
  pub fn new(source: &'source str) -> Self {
    Self {
      bytes: source.as_bytes(),
      spaces: 0,
      start: 0,
    }
  }

  pub fn bytes(&self) -> &'source [u8] {
    &self.bytes[self.start + self.spaces..]
  }

  pub fn forward_to(&mut self, size: usize) {
    self.spaces = 0;
    self.start = size;
  }

  pub fn forward(&mut self, size: usize) -> usize {
    self.start += size + self.spaces;
    self.spaces = 0;
    self.start
  }

  pub fn spaces(&self) -> usize {
    self.spaces
  }

  pub fn reset_spaces(&mut self) {
    self.spaces = 0;
  }

  pub fn spaces0(&mut self) -> usize {
    let (_, spaces) = spaces0(&self.bytes[self.start..]);
    self.spaces = spaces;
    spaces
  }

  pub fn start(&self) -> usize {
    self.start
  }
}
