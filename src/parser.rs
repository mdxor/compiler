struct Parser<'source> {
  source: &'source str,
  bytes: &'source [u8],
  index: usize,
}

impl<'source> Parser<'source> {
  fn new(source: &'source str) -> Self {
    Self {
      source,
      bytes: source.as_bytes(),
      index: 0,
    }
  }

  fn parse(&self) {}

  fn parse_blocks(&self) {}
}
