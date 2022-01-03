pub fn tag<'source>(bytes: &'source [u8], tag: &[u8]) -> Option<&'source [u8]> {
  let len = tag.len();
  if len > bytes.len() {
    None
  } else if &bytes[..len] == tag {
    Some(&bytes[len..])
  } else {
    None
  }
}

pub fn single_char(bytes: &[u8], ch: u8) -> Option<&[u8]> {
  if bytes.len() > 0 && bytes[0] == ch {
    Some(&bytes[1..])
  } else {
    None
  }
}

pub fn ch_repeat(bytes: &[u8], ch: u8) -> (&[u8], usize) {
  let repeat = bytes.iter().take_while(|&&c| c == ch).count();
  (&bytes[repeat..], repeat)
}

pub fn take_while<F>(bytes: &[u8], mut f: F) -> (&[u8], usize)
where
  F: FnMut(u8) -> bool,
{
  let size = bytes.iter().take_while(|&&c| f(c)).count();
  (&bytes[size..], size)
}

pub fn ch_repeat_min(bytes: &[u8], ch: u8, min: usize) -> Option<(&[u8], usize)> {
  let repeat = bytes.iter().take_while(|&&c| c == ch).count();
  if repeat >= min {
    Some((&bytes[repeat..], repeat))
  } else {
    None
  }
}

pub fn ch_repeat_max(bytes: &[u8], ch: u8, max: usize) -> Option<(&[u8], usize)> {
  let mut repeat = 0;
  for byte in bytes {
    if *byte == ch {
      repeat += 1;
      if repeat > max {
        return None;
      }
    } else {
      break;
    }
  }
  Some((&bytes[repeat..], repeat))
}

pub fn ch_repeat_min_max(bytes: &[u8], ch: u8, min: usize, max: usize) -> Option<(&[u8], usize)> {
  let mut repeat = 0;
  for byte in bytes {
    if *byte == ch {
      repeat += 1;
      if repeat > max {
        return None;
      }
    } else {
      break;
    }
  }
  if repeat >= min {
    Some((&bytes[repeat..], repeat))
  } else {
    None
  }
}

pub fn eol_or_space(bytes: &[u8]) -> Option<(&[u8], usize)> {
  if bytes.len() > 0 {
    match bytes[0] {
      b'\r' | b'\n' => Some((bytes, 0)),
      b' ' => Some((&bytes[1..], 1)),
      _ => None,
    }
  } else {
    Some((bytes, 0))
  }
}

pub fn spaces_eol(bytes: &[u8]) -> Option<(&[u8], usize)> {
  let mut size = 0;
  for byte in bytes {
    match byte {
      b'\r' => return Some((&bytes[size + 2..], size + 2)),
      b'\n' => return Some((&bytes[size + 1..], size + 1)),
      b' ' => {
        size += 1;
      }
      _ => {
        return None;
      }
    }
  }
  None
}

pub fn spaces0(bytes: &[u8]) -> (&[u8], usize) {
  take_while(bytes, |c| c == b' ')
}

pub fn eol(bytes: &[u8]) -> Option<(&[u8], usize)> {
  if bytes.len() > 0 {
    match bytes[0] {
      b'\r' => Some((&bytes[2..], 2)),
      b'\n' => Some((&bytes[1..], 1)),
      _ => None,
    }
  } else {
    Some((bytes, 0))
  }
}

// rest, size, size without eol
pub fn one_line(bytes: &[u8]) -> (usize, usize) {
  let (bytes, without_eol_size) = take_while(bytes, |c| c != b'\r' && c != b'\n');
  let (bytes, eol_size) = eol(bytes).unwrap();
  (without_eol_size + eol_size, without_eol_size)
}
