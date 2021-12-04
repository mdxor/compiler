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
}
