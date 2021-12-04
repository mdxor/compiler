pub struct Document<'source> {
  pub source: &'source str,
  pub bytes: &'source [u8],
  pub offset: usize,
  pub remaining: usize,
}

impl<'source> Document<'source> {
  pub fn new(source: &'source str) -> Self {
    Self {
      source,
      bytes: source.as_bytes(),
      offset: 0,
      remaining: 0,
    }
  }

  pub fn bytes(&self) -> &'source [u8] {
    &self.bytes[self.offset..]
  }

  pub fn source(&self) -> &'source str {
    &self.source[self.offset..]
  }
}
