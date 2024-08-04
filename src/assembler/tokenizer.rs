use crate::binary_utils::*;
use crate::assemble::Param;
use core::panic;


#[derive(Clone)]
pub struct SourceLine {
    pub text: String,
    pub number: u16,
    pub actual_line: u16,
}

impl SourceLine {
    pub fn new(text: &str, number: u16, actual_line: u16) -> Self {
        SourceLine {
            text: text.to_string(),
            number,
            actual_line,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Sign {
    PLUS = 1,
    MINUS = -1,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NumberLiteral {
    pub sign: Sign,
    pub value: u16,
    pub bits: u16,
}

impl NumberLiteral {
    pub fn new() -> Self {
        NumberLiteral {
            sign: Sign::PLUS,
            value: 0,
            bits: 0,
        }
    }
}

#[derive(Debug, Clone)]
//#[derive(PartialEq)]
pub enum Token {
    DecimalLiteral(NumberLiteral),
    HexLiteral(NumberLiteral),
    BinLiteral(NumberLiteral),
    Register(u16),
    Label(String),
    Comma,
    Instruction(String),
    Directive(String),
    StringLiteral(String),
    AlphabeticLblRegOrInstr,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.is(other)
    }
}

impl Token {
    pub fn tokenize_str(line: &str) -> Vec<Token> {
        Self::tokenize_line(&SourceLine::new(line, 0, 0)).unwrap()
    }

    pub fn is_directive(name: &str) -> bool {
        vec!["BLKW", "FILL", "ORIG", "END", "STRINGZ"].contains(&name)
    }

    pub fn is(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    pub fn is_valid_arg(&self, param: &Param) -> bool {
        match self {
            Self::Comma | Self::Directive(_) | Self::Instruction(_) | Self::StringLiteral(_) => {
                return false
            }
            _ => {}
        };

        match param {
            Param::Label /*| Param::Label6bit*/ => return self.is(&Token::Label(String::new())),

            Param::Register(_) => return self.is(&Token::Register(0)),

            Param::Bits(bits) => match self {
                Self::DecimalLiteral(number) | Self::HexLiteral(number) => {
                    return number.bits <= *bits - 1;
                }

                _ => return false,
            },

            // Param::Imm5 => {
            //     panic!("INTERNAL ERROR: \tUnexpected instr to be defined as having imm5.")
            // }

            Param::RegisterORImm5 => {
                return self.is(&Token::Register(0))
                    || match self {
                        Self::DecimalLiteral(number) | Self::HexLiteral(number) => {
                            number.bits <= 5 - 1
                        }
                        _ => false,
                    };
            }
        }
    }

    pub fn as_u16(&self, truncate_to: Option<u16>) -> u16 {
        if let Token::DecimalLiteral(number) | Token::HexLiteral(number) = self {
            let mut result = number.value;

            if matches!(number.sign, Sign::MINUS) {
                //println!("Negative imm5. {num}");
                result = invert_sign(number.value); /*+ binary_utils::flag_set_mask(5)*/
            }

            truncate_to.map(|limit| {
                if number.bits > limit {
                    panic!(
                        "{:?} is a {} bit number, expected a {} bit number.",
                        number, limit, number.bits
                    );
                }

                return truncate_to_bit(result, limit);
            });
            return result;
        } else {
            panic!("INTERNAL ERROR: {:?} is not a NumberLiteral.", &self);
        }

        // if truncate_to == 0 {
        //     result
        // } else {
        //     binary_utils::truncate_to(result, truncate_to)
        // }
    }

    pub fn tokenize_line(line: &SourceLine) -> Result<Vec<Token>, String> {
        let mut token_stream = Vec::new();

        let mut current_token: Option<Token> = None;
        let mut current_token_text = String::new();

        let mut index = 0;
        let mut line_as_chars: String = String::from(line.text.trim());
        line_as_chars.push('\n');

        for c in line_as_chars.chars() {
            // println!("{:?} = {}", current_token, current_token_text);
            index += 1;
            match current_token {
                None => {
                    if c == ' ' || c == '\n' || c == '\t' {
                        continue;
                    }

                    if c == ',' {
                        token_stream.push(Self::Comma);
                        continue;
                    }

                    if c == '"' {
                        current_token = Some(Self::StringLiteral(String::new()));
                        continue;
                    }

                    if c == '#' {
                        current_token = Some(Self::DecimalLiteral(NumberLiteral::new()));
                        continue;
                    }

                    if c == 'x' {
                        current_token = Some(Self::HexLiteral(NumberLiteral::new()));
                        continue;
                    }

                    if c == 'b' {
                        current_token = Some(Self::BinLiteral(NumberLiteral::new()));
                        continue;
                    }

                    if c == '.' {
                        current_token = Some(Self::Directive(String::new()));
                        continue;
                    }

                    if c == ';' {
                        println!("Warning: unexpected comment.");
                        break;
                        //continue;
                    }

                    if !c.is_ascii() {
                        return Err(format!("Invalid non-ASCII character '{}' ", c));
                    }else {
                        // unimplemented!("Refactor to merge current_token_text and current_token");
                        
                        if c.is_ascii_digit(){
                            current_token = Some(Self::DecimalLiteral(NumberLiteral::new()));
                        }else{
                            current_token = Some(Self::AlphabeticLblRegOrInstr);
                        }
                        

                        current_token_text.clear();
                        current_token_text.push(c);
                        continue;
                    }

                    

                    
                }

                Some(ref token) => {
                    match token {
                        Self::AlphabeticLblRegOrInstr => {
                            //Register
                            // if c.is_alphanumeric() {
                            //     current_token_text.push(c);
                            //     continue;
                            // }

                            //Terminators
                            if c == ',' || c == ' ' || c == '\n' || c == '\t' {
                                let text = &current_token_text;
                                //Register
                                if text.starts_with("R")
                                    && text.len() == 2
                                    && text.chars().nth(1).unwrap().is_ascii_digit()
                                {
                                    let register_no: u16 = text
                                        .chars()
                                        .nth(1)
                                        .unwrap()
                                        .to_string()
                                        .parse()
                                        .expect("Unable to parse register no.");

                                    if register_no < 8 {
                                        token_stream.push(Self::Register(register_no));

                                        if c == ',' {
                                            token_stream.push(Self::Comma);
                                        }
                                    } else {
                                        return Err(format!("Invalid register 'R{}'. Valid registers: R0, R1, ... R7.", register_no));
                                    }
                                }
                                //Instruction
                                else if crate::is_instruction(text) {
                                    token_stream.push(Self::Instruction(text.clone()));
                                }
                                //Label
                                else {
                                    token_stream.push(Self::Label(text.clone()));
                                }

                                current_token = None;
                                current_token_text.clear();
                            }else{
                                current_token_text.push(c);
                            }
                        }

                        Self::DecimalLiteral(_) => {
                            if !c.is_ascii_digit()
                                && current_token_text.len() != 0
                                && (c != '-')
                                && c != '\n'
                                && c != ' '
                            {
                                return Err(format!("Invalid decimal literal '{}'", c));
                            } else if c == '-' {
                                current_token_text.push(c);
                                continue;
                            }

                            if c.is_ascii_digit() {
                                current_token_text.push(c);
                                //println!("Found char {c} for decimal literal.");
                            } else if c == ',' || c == ' ' || c == '\n' || c == '\t' {
                                let mut interpretation: NumberLiteral = NumberLiteral::new();
                                //println!("'{}'", current_token_text);
                                if current_token_text.starts_with("-") {
                                    interpretation.sign = Sign::MINUS;
                                }
                                current_token_text = current_token_text
                                    .trim_start_matches('-')
                                    .to_string()
                                    .clone();
                                // println!("'{}'", current_token_text);
                                // break;
                                let value: u32 = match current_token_text.parse::<u32>() {
                                    Ok(val) => val,
                                    Err(e) => {
                                        return Err(format!("Invalid decimal: {e}."));
                                    }
                                };

                                if value > 2u32.pow(15) {
                                    return Err(format!(
                                        "Decimal literal {} is out of range. MAX = +/-{}",
                                        value,
                                        2u32.pow(15)
                                    ));
                                }

                                interpretation.value = value
                                    .try_into()
                                    .expect("Unable to convert decimal literal u32 to u16");

                                interpretation.bits =
                                    bits_required_for_number(interpretation.value);

                                token_stream.push(Self::DecimalLiteral(interpretation));
                                current_token_text.clear();

                                if c == ',' {
                                    token_stream.push(Self::Comma);
                                }
                            } else {
                                current_token_text.push(c);
                                return Err(format!("Invalid decimal: '#{current_token_text}'."));
                            }
                        }

                        Self::HexLiteral(_) => {
                            if !c.is_ascii_hexdigit()
                                && current_token_text.len() != 0
                                && (c != '-')
                                && c != '\n'
                            {
                                return Err(format!("Invalid hexdecimal literal."));
                            } else {
                                if c == '-' {
                                    current_token_text.push(c);
                                    continue;
                                };
                            }

                            if c.is_ascii_hexdigit() {
                                current_token_text.push(c);
                            } else if c == ',' || c == ' ' || c == '\n' || c == '\t' {
                                if index + 1 == line.text.len() {
                                    current_token_text.push(c);
                                }

                                let mut interpretation: NumberLiteral = NumberLiteral::new();

                                if current_token_text.starts_with("-") {
                                    interpretation.sign = Sign::MINUS;
                                }
                                let text_to_parse =
                                    current_token_text.trim_start_matches('-').to_string();
                                //println!("'{}'", current_token_text);
                                // break;
                                let value: u32 = match u32::from_str_radix(
                                    &text_to_parse.trim(),
                                    16,
                                ) {
                                    Ok(val) => val,
                                    Err(e) => {
                                        return Err(format!(
                                                "Invalid hexadecimal literal 'x{current_token_text}': {e}"
                                            ));
                                    }
                                };
                                if value > 0xffff {
                                    return Err(format!(
                                        "Hexadecimal literal {:0x} is out of range, MAX +/-0x{:0x}",
                                        value,
                                        2u32.pow(15)
                                    ));
                                }

                                interpretation.value = value
                                    .try_into()
                                    .expect("Unable to convert hexadecimal literal u32 to u16");

                                interpretation.bits =
                                    bits_required_for_number(interpretation.value);

                                token_stream.push(Self::HexLiteral(interpretation));
                                current_token_text.clear();

                                if c == ',' {
                                    token_stream.push(Self::Comma);
                                }
                            } else {
                                current_token_text.push(c);
                                return Err(format!(
                                    "Invalid hexadecimal: 'x{current_token_text}'."
                                ));
                            }
                        }

                        Self::BinLiteral(_) => {
                            if !(c == '0' || c == '1')
                                && current_token_text.len() != 0
                                && (c != '-')
                                && c != '\n'
                            {
                                return Err(format!("Invalid binary literal."));
                            } else {
                                if c == '-' {
                                    current_token_text.push(c);
                                    continue;
                                };
                            }

                            if c == '0' || c == '1'{
                                current_token_text.push(c);
                            } else if c == ',' || c == ' ' || c == '\n' || c == '\t' {
                                if index + 1 == line.text.len() {
                                    current_token_text.push(c);
                                }

                                let mut interpretation: NumberLiteral = NumberLiteral::new();

                                if current_token_text.starts_with("-") {
                                    interpretation.sign = Sign::MINUS;
                                }
                                let text_to_parse =
                                    current_token_text.trim_start_matches('-').to_string();
                                //println!("'{}'", current_token_text);
                                // break;
                                let value: u32 = match u32::from_str_radix(&text_to_parse.trim(), 2)
                                {
                                    Ok(val) => val,
                                    Err(e) => {
                                        return Err(format!(
                                            "Invalid binary literal 'b{current_token_text}': {e}"
                                        ));
                                    }
                                };
                                if value > 2u32.pow(15) {
                                    return Err(format!(
                                        "Binary literal b{:0b} is out of range, MAX +/-{}",
                                        value,
                                        2u32.pow(15)
                                    ));
                                }

                                interpretation.value = value
                                    .try_into()
                                    .expect("Unable to convert binary literal u32 to u16");

                                interpretation.bits =
                                    bits_required_for_number(interpretation.value);

                                token_stream.push(Self::HexLiteral(interpretation));
                                current_token_text.clear();

                                if c == ',' {
                                    token_stream.push(Self::Comma);
                                }
                            } else {
                                current_token_text.push(c);
                                return Err(format!("Invalid binary: 'x{current_token_text}'."));
                            }
                        }

                        Self::Directive(_) => {
                            if !c.is_ascii_alphabetic() {
                                if c == ',' || c == ' ' || c == '\n' || c == '\t' {
                                    if !Token::is_directive(&current_token_text) {
                                        return Err(format!(
                                            "'.{}' is not a valid directive.",
                                            current_token_text
                                        ));
                                    } else {
                                        token_stream
                                            .push(Self::Directive(current_token_text.clone()));
                                    }

                                    if c == ',' {
                                        token_stream.push(Self::Comma);
                                    }

                                    current_token = None;
                                    current_token_text.clear();
                                } else {
                                    current_token_text.push(c);
                                    return Err(format!(
                                        "Invalid directive '{}'",
                                        current_token_text
                                    ));
                                }
                            } else {
                                current_token_text.push(c);
                            }
                        }

                        Self::StringLiteral(_) => {
                            if c == '\n' {
                                return Err(format!("String literal, expected closing '\"'."));
                            }

                            if c == '"' {
                                token_stream.push(Self::StringLiteral(current_token_text.clone()));
                                current_token_text.clear();
                                current_token = None;
                            } else {
                                current_token_text.push(c);
                            }
                        }
                        _ =>
                            /*panic!()*/
                            {}
                    }
                }
            }
        }
        //println!("{}", current_token_text);

        Ok(token_stream)
    }
}