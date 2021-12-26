use crate::token::*;
pub struct Document<'source> {
  pub source: &'source str,
  pub bytes: &'source [u8],
  offset: usize,
  start: usize,
  pub link_definitions: Vec<LinkDefinition<'source>>,
}

impl<'source> Document<'source> {
  pub fn new(source: &'source str) -> Self {
    Self {
      source,
      bytes: source.as_bytes(),
      offset: 0,
      start: 0,
      link_definitions: vec![],
    }
  }

  pub fn bytes(&self) -> &'source [u8] {
    &self.bytes[self.offset..]
  }

  pub fn source(&self) -> &'source str {
    &self.source[self.offset..]
  }

  pub fn forward(&mut self, size: usize) -> usize {
    self.offset += size;
    self.start = self.offset;
    self.offset
  }

  pub fn forward_offset(&mut self, size: usize) -> usize {
    self.offset += size;
    self.offset
  }

  pub fn forward_to(&mut self, size: usize) {
    self.offset = size;
    self.start = size;
  }

  pub fn offset(&self) -> usize {
    self.offset
  }
  pub fn start(&self) -> usize {
    self.start
  }
}
