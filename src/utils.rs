use num_traits::Num;

#[derive(Clone)]
pub enum Command {
    Assemble,
    Run
}
impl Default for Command { fn default() -> Self { Self::Run } }
#[derive(Default)]
pub struct Options {
    pub command: Command,
    pub path: String,
    pub out_path: String,
    pub debug_level: i32
}

pub fn bytes_to_16(hb: u8, lb: u8) -> u16 {
    ((hb as u16) << 8) + lb as u16
}
pub fn u16_to_bytes(int: u16) -> (u8, u8) {
    ((int >> 8) as u8, (int & 255) as u8)
}
pub fn parse_int_literal<T: Num>(s: &str) -> Result<T, String> {
    match T::from_str_radix(s, 10) {
        Ok(v) => return Ok(v),
        Err(_) => {}
    }
    if s.len() < 3 {
        return Err(format!("literal {} too short", s))
    }
    let radix = &s[..2];
    let literal = &s[2..];
    Ok(match radix {
        "0b" => {
            match T::from_str_radix(literal, 2) {
                Ok(i) => i,
                Err(_) => return Err(format!("binary literal {} failed to parse", s))
            }
        }
        "0d" => {
            match T::from_str_radix(literal, 10) {
                Ok(i) => i,
                Err(_) => return Err(format!("decimal literal {} failed to parse", s))
            }
        }
        "0x" => {
            match T::from_str_radix(literal, 16) {
                Ok(i) => i,
                Err(_) => return Err(format!("hex literal {} failed to parse", s))
            }
        }
        _ => {
            return Err(format!("unsupported radix {}", radix))
        }
    })
}
pub fn set_vec_value_at_index<T: Default>(vec: &mut Vec<T>, val: T, index: usize) {
    while index >= vec.len() {
        vec.push(T::default())
    }
    vec[index] = val
}

pub fn strip_whitespace(s: &str) -> String {
    let mut ret = String::new();
    for c in s.chars() {
        if c != ' ' {
            ret.push(c)
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_conv_tests() {
        assert_eq!(u16_to_bytes(0b0000_1111_0000_0001), (0b0000_1111, 0b0000_0001));
        assert_eq!(bytes_to_16(0b0000_0011, 0b0011_0011), 0b0000_0011_0011_0011)
    }

    #[test]
    fn int_parse() {
        assert_eq!(parse_int_literal("0d123"), Ok(123));
        assert_eq!(parse_int_literal("0x123"), Ok(0x123));
        assert_eq!(parse_int_literal::<i32>("0b123"), Err(String::from("binary literal 0b123 failed to parse")));
        assert_eq!(parse_int_literal("123"), Ok(123));
    }
}