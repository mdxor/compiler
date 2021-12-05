pub struct Document<'source> {
  pub source: &'source str,
  pub bytes: &'source [u8],
  pub offset: usize,
  pub block_start: usize,
  pub remaining_spaces: usize,
}

impl<'source> Document<'source> {
  pub fn new(source: &'source str) -> Self {
    Self {
      source,
      bytes: source.as_bytes(),
      offset: 0,
      block_start: 0,
      remaining_spaces: 0,
    }
  }

  pub fn bytes(&self) -> &'source [u8] {
    &self.bytes[self.offset..]
  }

  pub fn source(&self) -> &'source str {
    &self.source[self.offset..]
  }
}
