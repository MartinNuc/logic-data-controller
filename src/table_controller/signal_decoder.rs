use std::collections::VecDeque;

// table height matching sequence
const HEIGHT_MATCHING_SEQUENCE: [u8; 23] = [
  1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
];

pub struct SignalDecoder {
  bits: VecDeque<u8>,
  pub current_height: Option<i32>,
}

impl SignalDecoder {
  pub fn new() -> SignalDecoder {
    SignalDecoder {
      bits: VecDeque::with_capacity(32),
      current_height: Option::None,
    }
  }

  pub fn process_bit(self: &mut Self, bit: u8) {
    if self.bits.len() >= 32 {
      self.bits.pop_front();
    }
    self.bits.push_back(bit);

    if self.is_matching_table_height_pattern() {
      self.update_current_height();
    }
  }

  fn is_matching_table_height_pattern(&self) -> bool {
    self
      .bits
      .iter()
      .take(23)
      .eq(HEIGHT_MATCHING_SEQUENCE.iter())
  }

  fn update_current_height(self: &mut Self) {
    let string_binary: String = self
      .bits
      .iter()
      .skip(23)
      .take(8)
      .rev()
      .map(|&x| if x > 0 { "0" } else { "1" })
      .collect::<String>();
    let new_height = isize::from_str_radix(&string_binary, 2).unwrap() as i32;

    // Plausibility check 1: height must be in a valid range
    if new_height < 60 || new_height > 120 {
      return;
    }

    // Plausibility check 2: height must not differ more than 5cm from last value
    if self.current_height.is_none() || (self.current_height.unwrap() - new_height).abs() < 5 {
      self.current_height = Some(new_height);
    }
  }
}
