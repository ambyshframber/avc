#[derive(Clone)]
pub enum Command {
    Assemble,
    Run
}
impl Default for Command { fn default() -> Self { Self::Run } }
#[derive(Default)]
pub struct Options {
    pub command: Command,
    pub path: String
}

pub fn bytes_to_16(hb: u8, lb: u8) -> u16 {
    ((hb as u16) << 8) + lb as u16
}
pub fn u16_to_bytes(int: u16) -> (u8, u8) {
    ((int >> 8) as u8, (int & 255) as u8)
}
pub fn parse_int_literal(s: &str) -> Result<u16, String> {
    let radix = &s[..2];
    let literal = &s[2..];
    Ok(match radix {
        "0b" => {
            match u16::from_str_radix(literal, 2) {
                Ok(i) => i,
                Err(_) => return Err(format!("binary literal {} failed to parse", s))
            }
        }
        "0d" => {
            match u16::from_str_radix(literal, 10) {
                Ok(i) => i,
                Err(_) => return Err(format!("decimal literal {} failed to parse", s))
            }
        }
        "0x" => {
            match u16::from_str_radix(literal, 16) {
                Ok(i) => i,
                Err(_) => return Err(format!("hex literal {} failed to parse", s))
            }
        }
        _ => {
            return Err(format!("unsupported radix {}", radix))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_conv_tests() {
        assert_eq!(u16_to_bytes(0b0000_1111_0000_0001), (0b0000_1111, 0b0000_0001));
        assert_eq!(bytes_to_16(0b0000_0011, 0b0011_0011), 0b0000_0011_0011_0011)
    }
}