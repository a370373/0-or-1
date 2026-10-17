pub fn parse_bits(input: &str) -> Result<Vec<u8>, String> {
    let bytes = input.as_bytes();
    let mut output = Vec::new();

    let mut current_byte: u8 = 0;
    let mut bits_in_byte: u8 = 0;
    let mut total_bits: usize = 0;
    let mut offset: usize = 0;

    while offset < bytes.len() {
        let byte = bytes[offset];

        // '#' 開始行註解，忽略直到換行符號。
        if byte == b'#' {
            while offset < bytes.len() && bytes[offset] != b'\n' {
                offset += 1;
            }
            continue;
        }

        match byte {
            b'0' | b'1' => {
                current_byte = (current_byte << 1) | (byte - b'0');
                bits_in_byte += 1;
                total_bits += 1;

                if bits_in_byte == 8 {
                    output.push(current_byte);
                    current_byte = 0;
                    bits_in_byte = 0;
                }
            }

            b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C => {
                // 忽略 ASCII 空白字元。
            }

            _ => {
                return Err(format!(
                    "invalid byte {:?} at offset {}; \
                     only 0, 1, ASCII whitespace, and # comments are allowed",
                    byte as char, offset
                ));
            }
        }

        offset += 1;
    }

    if total_bits % 8 != 0 {
        return Err(format!(
            "incomplete byte: total bit count {} is not divisible by 8",
            total_bits
        ));
    }

    if output.is_empty() {
        return Err("input contains no machine-code bytes".to_string());
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::parse_bits;

    #[test]
    fn parses_one_byte() {
        assert_eq!(parse_bits("01000001").unwrap(), vec![0x41]);
    }

    #[test]
    fn accepts_whitespace() {
        assert_eq!(
            parse_bits("01000001 01000010\n01000011\t01000100").unwrap(),
            vec![0x41, 0x42, 0x43, 0x44]
        );
    }

    #[test]
    fn accepts_full_line_comments() {
        assert_eq!(
            parse_bits("# first comment\n01000001\n# second comment\n01000010").unwrap(),
            vec![0x41, 0x42]
        );
    }

    #[test]
    fn accepts_inline_comments() {
        assert_eq!(
            parse_bits(
                "01000001 # letter A\n\
                 01000010 # letter B\n"
            )
            .unwrap(),
            vec![0x41, 0x42]
        );
    }

    #[test]
    fn comments_can_contain_binary_digits() {
        assert_eq!(
            parse_bits("01000001 # 11111111 00000000\n").unwrap(),
            vec![0x41]
        );
    }

    #[test]
    fn ignores_blank_lines_and_indentation() {
        assert_eq!(
            parse_bits("\n  # comment\n    01000001\n\n\t01000010\n").unwrap(),
            vec![0x41, 0x42]
        );
    }

    #[test]
    fn rejects_incomplete_byte() {
        assert!(parse_bits("0100000").is_err());
    }

    #[test]
    fn rejects_non_binary_characters() {
        assert!(parse_bits("0100000x").is_err());
    }

    #[test]
    fn parses_zero_byte() {
        assert_eq!(parse_bits("00000000").unwrap(), vec![0x00]);
    }

    #[test]
    fn rejects_comment_only_input() {
        assert!(parse_bits("# nothing to execute\n").is_err());
    }
}
