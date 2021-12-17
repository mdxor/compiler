pub struct Document<'source> {
  source: &'source str,
  bytes: &'source [u8],
  offset: usize,
  _offset: usize,
}

impl<'source> Document<'source> {
  pub fn new(source: &'source str) -> Self {
    Self {
      source,
      bytes: source.as_bytes(),
      offset: 0,
      _offset: 0,
    }
  }

  pub fn bytes(&self) -> &'source [u8] {
    &self.bytes[self._offset..]
  }

  pub fn source(&self) -> &'source str {
    &self.source[self._offset..]
  }

  pub fn forward(&mut self, size: usize) -> usize {
    self._offset += size;
    self.offset = self._offset;
    self._offset
  }

  pub fn forward_for_next(&mut self, size: usize) -> usize {
    self._offset += size;
    self._offset
  }

  pub fn offset(&self) -> usize {
    self.offset
  }
}
