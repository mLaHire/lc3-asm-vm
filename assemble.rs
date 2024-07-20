use crate::binary_utils;
use crate::binary_utils::flag_set_mask;
use crate::error;
use crate::virtual_machine;
use core::panic;
use io::BufRead;
use std::fs::File;
use std::io;
use std::io::BufReader;

#[derive(Debug)]
pub struct Symbol {
    name: String,
    offset_from_origin: u16,
    size_in_words: u16,
}

#[derive(PartialEq, Debug, Clone)]
enum Sign {
    PLUS = 1,
    MINUS = -1,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NumberLiteral {
    sign: Sign,
    value: u16,
    bits: u16,
}

impl NumberLiteral {
    fn new() -> Self {
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
            Param::Label | Param::Label6bit => return self.is(&Token::Label(String::new())),

            Param::Register(_) => return self.is(&Token::Register(0)),

            Param::Bits(bits) => match self {
                Self::DecimalLiteral(number) | Self::HexLiteral(number) => {
                    return number.bits <= *bits - 1;
                }

                _ => return false,
            },

            Param::Imm5 => {
                panic!("INTERNAL ERROR: \tUnexpected instr to be defined as having imm5.")
            }

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
                result = binary_utils::invert_sign(number.value); /*+ binary_utils::flag_set_mask(5)*/
            }

            truncate_to.map(|limit| {
                if number.bits > limit {
                    panic!(
                        "{:?} is a {} bit number, expected a {} bit number.",
                        number, limit, number.bits
                    );
                }

                return binary_utils::truncate_to_bit(result, limit);
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

                    if c == '.' {
                        current_token = Some(Self::Directive(String::new()));
                        continue;
                    }

                    if c.is_ascii_alphabetic() {
                        // unimplemented!("Refactor to merge current_token_text and current_token");
                        current_token = Some(Self::AlphabeticLblRegOrInstr);

                        current_token_text.clear();
                        current_token_text.push(c);
                        continue;
                    }

                    if c == ';' {
                        println!("Warning: unexpected comment.");
                        break;
                        //continue;
                    }

                    return Err(format!("Invalid character '{}' ", c));
                }

                Some(ref token) => {
                    match token {
                        Self::AlphabeticLblRegOrInstr => {
                            //Register
                            if c.is_alphanumeric() {
                                current_token_text.push(c);
                                continue;
                            }

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
                                else if is_instruction(text) {
                                    token_stream.push(Self::Instruction(text.clone()));
                                }
                                //Label
                                else {
                                    token_stream.push(Self::Label(text.clone()));
                                }

                                current_token = None;
                                current_token_text.clear();
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

                                if value > 0xFFFF {
                                    return Err(format!(
                                        "Decimal literal {} is out of range.",
                                        value
                                    ));
                                }

                                interpretation.value = value
                                    .try_into()
                                    .expect("Unable to convert decimal literal u32 to u16");

                                interpretation.bits =
                                    binary_utils::bits_required_for_number(interpretation.value);

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
                                current_token_text =
                                    current_token_text.trim_start_matches('-').to_string();
                                //println!("'{}'", current_token_text);
                                // break;
                                let value: u32 =
                                    u32::from_str_radix(&current_token_text, 16).unwrap();
                                if value > 0xFFFF {
                                    return Err(format!(
                                        "Hexadecimal literal {:0x} is out of range.",
                                        value
                                    ));
                                }

                                interpretation.value = value
                                    .try_into()
                                    .expect("Unable to convert hexadecimal literal u32 to u16");

                                interpretation.bits =
                                    binary_utils::bits_required_for_number(interpretation.value);

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

#[derive(Clone)]
pub struct SourceLine {
    pub text: String,
    pub number: u16,
    pub actual_line: u16,
}

impl SourceLine {
    fn new(text: &str, number: u16, actual_line: u16) -> Self {
        SourceLine {
            text: text.to_string(),
            number,
            actual_line,
        }
    }
}

pub struct TrapInstruction {
    instructions: Vec<u16>,
    origin: u16,
    trap_vector: u16,
    memory_writes: Vec<(u16, u16)>,
}

impl TrapInstruction {
    pub fn new(filename: &str, trap_vector: u16) -> Self {
        let mut asm = Assembler::new(format!(".\\src\\asm-files\\trap\\{}.asm", filename).as_str());
        asm.load();
        asm.tokenize();
        match asm.parse_origin_and_end() {
            Err(e) => panic!("Error finding program .ORIG and .END: {e}"),
            Ok(r) => println!("TRAP Program\t.ORIG {:x}\t.END{:x}", r.0, r.1),
        };
        asm.load_symbols();
        //asm.trim_lines();
        let memory_writes = asm.parse_directives_to_list();
        let instructions = asm.parse_instructions();
        TrapInstruction {
            memory_writes,
            instructions,
            trap_vector,
            origin: asm.orig,
        }
    }
}

pub struct Assembler {
    file_path: String,
    raw_lines: Vec<String>,
    processed_lines: Vec<SourceLine>,
    tokenized_lines: Vec<(Vec<Token>, u16)>,
    symbol_table: SymbolTable,
    instruction_set: HashMap<String, InstrDef>,
    vm: virtual_machine::VirtualMachine,
    pub orig: u16,
    end: u16,
}

impl Assembler {}

impl Assembler {
    pub fn new(path: &str) -> Self {
        Self {
            file_path: path.to_string(),
            raw_lines: Vec::new(),
            processed_lines: Vec::new(),
            tokenized_lines: Vec::new(),
            symbol_table: Vec::new(),
            instruction_set: Parser::define_instruction_set(),
            vm: virtual_machine::VirtualMachine::new(),
            orig: 0,
            end: 0,
        }
    }

    pub fn load(&mut self) {
        let file_open_result = File::open(self.file_path.as_str());

        let file = match file_open_result {
            Ok(f) => f,
            Err(e) => {
                dbg!(e);
                panic!("{:?}", error::FileLoadError::FsOpenFailed);
            }
        };

        let reader = BufReader::new(file);

        let file_read_result: Vec<String> = reader
            .lines()
            .map(|l| l.expect("Failure to read line"))
            .collect();

        self.raw_lines = file_read_result;
        self.processed_lines = self.omit_comments();
        for ln in &self.processed_lines {
            println!("{:03}\t{}", ln.actual_line, ln.text);
        }
        //self.symbol_table = Self::build_symbol_table(&self.processed_lines);
        //println!("Symbol table: {:#?}", self.symbol_table);
    }

    pub fn omit_comments(&self) -> Vec<SourceLine> {
        let mut result: Vec<SourceLine> = Vec::new();

        let mut non_comment_line_found = false;
        let mut skip_count: u16 = 0;

        if self.raw_lines.len() == 0 {
            panic!("Cannot omit comments without loading file.");
        }

        for i in 0..self.raw_lines.len() {
            if self.raw_lines[i].trim_start().starts_with(";") && !non_comment_line_found {
                skip_count += 1;
                continue;
            }

            non_comment_line_found = true;

            let mut line = String::new();
            for char in self.raw_lines[i].chars() {
                if char == ';' {
                    //println!("Comment found.");
                    break;
                }

                line.push(char);
            }
            let n: u16 = i.try_into().unwrap();

            if line.is_empty() {
                continue;
            }

            result.push(SourceLine::new(&line.clone(), n - skip_count, n + 1));
        }
        result
    }

    pub fn build_symbol_table(lines: &Vec<SourceLine>) -> SymbolTable {
        todo!();
        let mut table: SymbolTable = Vec::new();

        for line in lines {
            parse_line_for_symbol(&line).map(|name| {
                if table
                    .iter()
                    .map(|s| s.name.clone())
                    .collect::<Vec<String>>()
                    .contains(&name)
                {
                    panic!("ASM ERROR: Symbol {} defined twice.", name);
                }
                table.push(Symbol {
                    name: name.clone(),
                    offset_from_origin: line
                        .number
                        .try_into()
                        .expect("Unable to convert usize->u16"),
                    size_in_words: 1,
                });
            });
        }

        table
    }

    pub fn load_symbols(&mut self) {
        self.trim_lines();
        for (token_stream, number) in &self.tokenized_lines {
            if let Token::Label(symbol) = token_stream.first().unwrap() {
                self.symbol_table.push(Symbol {
                    name: symbol.clone(),
                    offset_from_origin: *number,
                    size_in_words: 1,
                });
            }
        }
        println!("Symbol table: {:#?}", self.symbol_table);
    }

    pub fn tokenize(&mut self) {
        for ln in &self.processed_lines {
            let token_stream = match Token::tokenize_line(ln) {
                Ok(tk) => tk,
                Err(e) => {
                    eprintln!(
                        "\nSyntax error ('{}' (line {})):\n\n{:02}\t\t'{}'\n\n\t\t{}",
                        self.file_path, ln.actual_line, ln.actual_line, ln.text, e
                    );
                    panic!();
                }
            };

            println!("{:02}     {:?}", ln.actual_line, token_stream);
            self.tokenized_lines.push((token_stream, ln.number));
        }
    }

    pub fn trim_lines(&mut self) {
        let filtered = self
            .tokenized_lines
            .clone()
            .into_iter()
            .filter(|tk| !(tk.0.is_empty() || tk.0.starts_with(&[Token::Directive(String::new())])))
            .enumerate();

        self.tokenized_lines.clear();
        for (index, line) in filtered {
            self.tokenized_lines.push((line.0, index as u16));
        }
    }

    // pub fn parse(&mut self) {
    //     let instruction_set = Parser::define_instruction_set();
    // }

    pub fn parse_origin_and_end(&mut self) -> Result<(u16, u16), String> {
        let mut found_orig = false;
        let mut expecting_origin_value_next = false;

        let mut found_end = false;

        //let expected_origin_tk = &self.tokenized_lines.first().expect("Expected origin.").0.first().expect("Expected token");

        for ln in &self.tokenized_lines {
            let token_stream = &ln.0;

            println!("{:03}\t{:?}", ln.1, ln.0);

            for token in token_stream {
                match token {
                    Token::Directive(dir) => {
                        if dir != "ORIG" {
                            if !found_orig {
                                return Err(String::from(
                                    "Expected .ORIG directive. Found directive '.{dir}' instead.",
                                ));
                            }
                            /*else if dir != "END" && !found_end{
                                return Err(format!("Expected .END directive. Found directive '.{dir}' instead."));
                            }*/
                            else if dir == "END" {
                                if found_end {
                                    return Err(format!(".END already defined ({:x}).", self.end));
                                }

                                self.end = ln.1 + self.orig;
                                found_end = true;
                            }
                        } else {
                            if found_orig {
                                return Err(format!(".ORIG aleady defined ({}).", self.orig));
                            } else {
                                expecting_origin_value_next = true;
                            }
                        }
                    }

                    Token::DecimalLiteral(val) | Token::HexLiteral(val) => {
                        if !expecting_origin_value_next && !found_orig {
                            return Err(format!("Not expecting decimal literal."));
                        } else {
                            if !found_orig {
                                match val.sign {
                                    Sign::MINUS => {
                                        return Err(format!(
                                            ".ORIG must be set to a positive value."
                                        ))
                                    }
                                    _ => {}
                                }

                                self.orig = val.value;
                                found_orig = true;
                                expecting_origin_value_next = false;
                            }
                        }
                    }

                    other => {} /*return Err(if !expecting_origin_value_next {format!("Expected .orig directive. Found {:?} ", other)} else {format!("Expecting number literal.")})*/,
                }
            }
        }

        if !found_orig {
            return Err(format!("Unable to find .ORIG"));
        }

        if !found_end {
            return Err(format!("Unable to find .END"));
        }

        Ok((self.orig, self.end))
    }

    pub fn parse_directives(&mut self) {
        let mut reserved_word_count = 0u16;

        for (line, line_offset) in &self.tokenized_lines {
            let line = match line.strip_prefix(&[Token::Label(format!(""))]) {
                Some(without_label) => without_label,
                None => line,
            };
            //println!("{line_offset} \t {line:?}");
            let mut line = line.iter().take(2);
            match match line.next() {
                Some(val) => val,
                None => continue,
            } {
                &Token::Directive(ref directive) => {
                    println!("Found directive {line:?} '{directive}'");
                    if directive != "ORIG"
                        && directive != "END"
                        && *line_offset < (self.end - self.orig)
                    {
                        println!(
                            "WARNING: Expected .END before directive .{directive} line offset = {}",
                            *line_offset
                        );
                    }

                    let unadjusted_offset = *line_offset;

                    let line_offset = line_offset + reserved_word_count;

                    if directive == "FILL" {
                        match line.next() {
                            Some(token) => {
                                println!("{:?}", token);
                                if token.is(&Token::HexLiteral(NumberLiteral::new()))
                                    || token.is(&Token::DecimalLiteral(NumberLiteral::new()))
                                {
                                    self.vm
                                        .write_memory(self.orig + line_offset, token.as_u16(None));
                                } else {
                                    panic!("NaN");
                                }
                            }
                            None => println!("Empty."),
                        }
                    } else if directive == "STRINGZ" {
                        match line.next() {
                            Some(token) => {
                                println!("{:?}", token);
                                if let Token::StringLiteral(text) = token {
                                    if !text.is_ascii() {
                                        panic!(
                                            "StringLiteral '{text}' contains non-ASCII characters."
                                        );
                                    }

                                    for (i, ch) in text.bytes().enumerate() {
                                        self.vm.write_memory(
                                            self.orig + line_offset + (i as u16),
                                            ch as u16,
                                        );
                                    }

                                    //Null term
                                    self.vm.write_memory(
                                        self.orig + line_offset + (text.bytes().len() as u16),
                                        0,
                                    );
                                    reserved_word_count += text.bytes().len() as u16;
                                    for sym in &mut self.symbol_table {
                                        if sym.offset_from_origin == unadjusted_offset {
                                            sym.size_in_words = 1 + text.bytes().len() as u16;
                                        }
                                    }
                                } else {
                                    panic!("NaN");
                                }
                            }
                            None => println!("Empty."),
                        }
                    } else if directive == "BLKW" {
                        match line.next() {
                            Some(token) => {
                                println!("{:?}", token);
                                if token.is(&Token::HexLiteral(NumberLiteral::new()))
                                    || token.is(&Token::DecimalLiteral(NumberLiteral::new()))
                                {
                                    reserved_word_count += token.as_u16(None);
                                    println!("Reserving {} words", token.as_u16(None));
                                } else {
                                    panic!("BLKW... NaN");
                                }
                            }
                            None => println!("BLKW empty."),
                        }
                    }
                }

                _ => (),
            };
            // let param = match line.next() {
            //     Some(val) => val,
            //     None => break,
            // };
        }

        // for ln in &self.tokenized_lines {
        //     let token_stream = &ln.0;
        // }
    }

    pub fn adjust_symbols(&mut self) {
        let mut cummulative_offset = 0;
        for symbol in &mut self.symbol_table {
            symbol.offset_from_origin += cummulative_offset;
            if symbol.size_in_words > 1 {
                cummulative_offset += symbol.size_in_words - 1;
            }
        }
        println!("Adjusted symbol table: {:#?}", self.symbol_table);
    }

    pub fn parse_directives_to_list(&mut self) -> Vec<(u16, u16)> {
        let mut memory_writes = Vec::new();
        let mut reserved_word_count = 0u16;
        let mut skip_count = 0;

        for (line, line_offset) in &self.tokenized_lines {
            let line = match line.strip_prefix(&[Token::Label(format!(""))]) {
                Some(without_label) => without_label,
                None => line,
            };
            //println!("{line_offset} \t {line:?}");
            let mut line = line.iter().take(2);
            match match line.next() {
                Some(val) => val,
                None => continue,
            } {
                &Token::Directive(ref directive) => {
                    println!("Found directive .{directive} {:0x}+{line_offset:0x} 0x{:03x} '{directive}'", self.orig, self.orig + line_offset);
                    if directive != "ORIG"
                        && directive != "END"
                        && *line_offset < (self.end - self.orig - 2)
                    {
                        panic!("SYNTAX ERROR: Expected .END before directive .{directive} line offset = {}", *line_offset);
                    }

                    let line_offset = line_offset + reserved_word_count;

                    if directive == "FILL" {
                        match line.next() {
                            Some(token) => {
                                println!("{:?}", token);
                                if token.is(&Token::HexLiteral(NumberLiteral::new()))
                                    || token.is(&Token::DecimalLiteral(NumberLiteral::new()))
                                {
                                    memory_writes
                                        .push((self.orig + line_offset, token.as_u16(None)));
                                } else {
                                    panic!("NaN");
                                }
                            }
                            None => println!("Empty."),
                        }
                    } else if directive == "STRINGZ" {
                        match line.next() {
                            Some(token) => {
                                println!("{:?}", token);
                                if let Token::StringLiteral(text) = token {
                                    if !text.is_ascii() {
                                        panic!(
                                            "StringLiteral '{text}' contains non-ASCII characters."
                                        );
                                    }

                                    for (i, ch) in text.bytes().enumerate() {
                                        memory_writes.push((
                                            self.orig + line_offset + (i as u16),
                                            ch as u16,
                                        ));
                                    }

                                    //Null term
                                    memory_writes.push((
                                        self.orig + line_offset + (text.bytes().len() as u16),
                                        0,
                                    ));
                                    reserved_word_count += text.bytes().len() as u16;
                                } else {
                                    panic!("NaN");
                                }
                            }
                            None => println!("Empty."),
                        }
                    } else if directive == "BLKW" {
                        match line.next() {
                            Some(token) => {
                                println!("{:?}", token);
                                if token.is(&Token::HexLiteral(NumberLiteral::new()))
                                    || token.is(&Token::DecimalLiteral(NumberLiteral::new()))
                                {
                                    reserved_word_count += token.as_u16(None);
                                    println!("Reserving {} words", token.as_u16(None));
                                } else {
                                    panic!("BLKW... NaN");
                                }
                            }
                            None => println!("BLKW empty."),
                        }
                    } else {
                        skip_count += 1;
                    }
                }

                _ => (skip_count += 1),
            };
        }
        //memory_writes =  memory_writes.into_iter().map(|w|(w.0-1,w.1)).collect();
        println!("MEM_WRITES: {:?}", memory_writes);
        memory_writes
    }

    pub fn parse_instructions_then_run(&mut self, trap_instructions: Option<Vec<TrapInstruction>>) {
        let mut instructions: Vec<u16> = Vec::new();

        //println!("\n Removing leading labels.");
        for (line, line_offset) in &self.tokenized_lines {
            let line = match line.strip_prefix(&[Token::Label(format!(""))]) {
                Some(without_label) => without_label,
                None => line,
            };
            //println!("{line_offset} \t {line:?}");
            match self.parse_single_instr(line.to_vec(), *line_offset) {
                None => {}
                Some(word) => {
                    instructions.push(word);
                }
            }
        }

        // let filtered_lines = (&self.tokenized_lines)
        //     .into_iter()
        //     .map(|(line, line_offset)| {line.into_iter().map())});
        let vm = &mut self.vm;
        vm.set_program_counter(self.orig);

        trap_instructions.map(|x| {
            for trap in x {
                vm.write_memory(trap.trap_vector, trap.origin);
                for (addr, val) in trap.memory_writes {
                    vm.write_memory(addr, val);
                }
                vm.load_binary_into_memory(trap.instructions, trap.origin);
            }
        });

        vm.load_binary_into_memory(instructions, self.orig);

        vm.run_io_thread();
        loop {
            vm.fetch();
            vm.decode();
            vm.execute();

            if !vm.run {
                break;
            }
            //Term::stdout().read_char();
        }
    }

    pub fn parse_instructions(&mut self) -> Vec<u16> {
        let mut instructions: Vec<u16> = Vec::new();

        //println!("\n Removing leading labels.");
        for (line, line_offset) in &self.tokenized_lines {
            let line = match line.strip_prefix(&[Token::Label(format!(""))]) {
                Some(without_label) => without_label,
                None => line,
            };
            //println!("{line_offset} \t {line:?}");
            match self.parse_single_instr(line.to_vec(), *line_offset) {
                None => {}
                Some(word) => {
                    instructions.push(word);
                }
            }
        }

        // let filtered_lines = (&self.tokenized_lines)
        //     .into_iter()
        //     .map(|(line, line_offset)| {line.into_iter().map())});
        return instructions;
    }

    fn parse_single_instr(&self, line: Vec<Token>, line_offset: u16) -> Option<u16> {
        let target_instruction: &InstrDef; // = &InstrDef::new(OP::RES, 0, vec![]);

        if line.starts_with(&[Token::Instruction(format!(""))]) {
            //Find which instruction it is;
            match line.first().unwrap() {
                Token::Instruction(instr) => {
                    //println!("{:03} Searching for instruction '{instr}'.", line_offset);
                    target_instruction = &self.instruction_set[instr]; //.expect("Undefined instruction {instr}");
                }
                // /Token::Directive(dir) => {
                //     println!("Ignoring directive .{dir} (line {line_offset})");
                // },
                _ => panic!("Expected instr, fatal error."),
            }
        } else {
            //println!("Ignoring line {line:?}");
            return None;
        }

        //Get line's paramaters
        let mut args: Vec<Token> = vec![];
        let mut arg_index = 0;

        //let mut previous: Vec<Token> = Vec::new();

        for i in 0..line.len() {
            if i == 0 {
                //Ignore instruction
                continue;
            }

            // println!("Token: '{:?}', i = {i}, arg_index = {arg_index}", line[i]);

            if arg_index == target_instruction.params.len() {
                panic!(
                    "Expected {} operands, found {}. \t[{:?}]",
                    target_instruction.params.len(),
                    arg_index,
                    line
                );
            }

            if (i % 2) == 0 && !line[i].is(&Token::Comma) {
                panic!("Expecting comma between params. {:?}", line);
            }

            if line[i].is(&Token::Comma) {
                continue;
            }

            if i % 2 != 0 && !line[i].is_valid_arg(&target_instruction.params[arg_index]) {
                panic!(
                    "Expected {:?} for instruction '{}', found token '{:?}'",
                    target_instruction.params[arg_index], target_instruction.opcode, line[i]
                );
            }

            //previous.push(line[i].clone());

            args.push(line[i].clone());
            arg_index += 1;
        }

        //println!("{line_offset}\t{args:?}\t{target_instruction:?}");

        let mut word: u16 = 0;
        word += target_instruction.opcode;
        word += target_instruction.flags_word;

        //
        // RERWRITE to avoid indexing
        //

        if target_instruction.params.len() != args.len() {
            panic!(
                "ERROR: Expected {} arguments, but found {}.",
                target_instruction.params.len(),
                args.len()
            );
        }
        for k in 0..target_instruction.params.len() {
            match target_instruction.params[k] {
                Param::Register(pos) => match args[k] {
                    Token::Register(r) => word += r << pos,
                    _ => panic!(),
                },

                Param::RegisterORImm5 => {
                    match &args[k] {
                        Token::DecimalLiteral(val) | Token::HexLiteral(val) => {
                            let mut num: u16 = val.value;
                            println!("|Imm5|: {num:016b} ({num}))");
                            if num > 2u16.pow(4) {
                                panic!("Invalid imm5");
                            }

                            if matches!(val.sign, Sign::MINUS) {
                                println!("Negative imm5. {num}");
                                num = binary_utils::truncate_to_bit(binary_utils::invert_sign(num), 5) /*+ binary_utils::flag_set_mask(5)*/;
                            }
                            word += num;
                            word = binary_utils::set_flag_true(word, 5);
                            //word += binary_utils::flag_set_mask(5);
                        }
                        Token::Register(reg) => {
                            word += reg;
                        }
                        _ => panic!(),
                    }
                }

                Param::Bits(bits) => {
                    //match &args[k] {
                    if let Token::DecimalLiteral(val) | Token::HexLiteral(val) = &args[k] {
                        let mut num: u16 = val.value;
                        if matches!(val.sign, Sign::MINUS) {
                            num =
                                binary_utils::truncate_to_bit(binary_utils::invert_sign(num), bits);
                        }

                        word += num;
                    }
                    // };
                    // if let Token::Label(s) = &args[k]{

                    // }
                    //     _ => panic!(),
                    // }
                }

                Param::Label => {
                    match &args[k] {
                        Token::Label(lbl) => {
                            let mut symbol_value: Option<u16> = None;

                            for sym in &self.symbol_table {
                                if sym.name == *lbl {
                                    symbol_value = Some(sym.offset_from_origin);
                                }
                            }

                            if symbol_value == None {
                                panic!("Undefined label '{lbl}'");
                            }
                            let symbol_value = symbol_value.unwrap();

                            //PC-Offset-9
                            println!(
                                "Label: {symbol_value}, PC: {line_offset}. L-PC = {}",
                                binary_utils::as_negative_i16(binary_utils::add_2s_complement(
                                    symbol_value,
                                    binary_utils::invert_sign(line_offset)
                                ))
                            );

                            let mut pc_offset_9 = binary_utils::truncate_to_n_bit(binary_utils::add_2s_complement(
                                    symbol_value,
                                    binary_utils::invert_sign(line_offset+1),),10) /*<< 7
                                    >> 7*/
                            ;
                            //WARNING

                            // if binary_utils::is_negative(pc_offset_9){
                            //     pc_offset_9 = (binary_utils::set_flag_true(pc_offset_9, 8) << 7 ) >> 7;
                            // }

                            // println!(
                            //     "PC-offset 9 = {}",
                            //     binary_utils::as_negative_i16(pc_offset_9)
                            // );
                            word += pc_offset_9;
                        }

                        _ => panic!(),
                    }
                }
                _ => {}
            }
        }

        println!("Instr: {word:016b}");
        return Some(word);
    }
}

pub type SymbolTable = Vec<Symbol>;

fn parse_line_for_symbol(line: &SourceLine) -> Option<String> {
    //First remove comments
    //let mut contains_comment = false;

    let mut ln = String::new();

    for ch in line.text.chars() {
        if ch == ';' {
            break;
        }
        ln.push(ch);
    }

    if ln.len() == 0 {
        return None;
    }

    let first_part = match ln.split_whitespace().next() {
        None => return None,
        Some(s) => s.trim(),
    };

    if first_part.is_empty() || first_part.starts_with(".") || is_instruction(first_part) {
        return None;
    }

    match first_part.chars().next() {
        Some(c) => {
            if c.is_numeric() || c == 'x' || !c.is_alphabetic() {
                return None;
            }
        }
        None => {
            panic!("Error");
        }
    }

    Some(String::from(first_part))
}

fn is_instruction(s: &str) -> bool {
    vec![
        "AND", "ADD", "NOT", "BR", "BRz", "BRp", "BRn", "BRnz", "BRnzp", "BRnp", "BRzp", "LD",
        "LDI", "LDR", "ST", "STR", "STI", "TRAP", "JMP", "RET", "JSR", "JSRR", "LEA", "HALT",
    ]
    .contains(&s)
}

use std::collections::HashMap;

#[derive(Debug)]
struct InstrDef {
    opcode: u16,
    flags_word: u16,
    params: Vec<Param>,
}

impl InstrDef {
    fn new(opcode: virtual_machine::OP, flags_word: u16, params: Vec<Param>) -> Self {
        InstrDef {
            opcode: (opcode as u16) << 12,
            flags_word,
            params,
        }
    }
}

struct Parser {
    // lines: Vec<String>,
    // symbolTable: SymbolTable,
    // instruction_set: HashMap<String, InstrDef>,
}
use virtual_machine::OP;

impl Parser {
    fn define_instruction_set() -> HashMap<String, InstrDef> {
        let mut instr_set = HashMap::new();
        instr_set.insert(
            String::from("ADD"),
            InstrDef::new(
                OP::ADD,
                0,
                vec![
                    Param::Register(9),
                    Param::Register(6),
                    Param::RegisterORImm5,
                ],
            ),
        );
        instr_set.insert(
            String::from("AND"),
            InstrDef::new(
                OP::AND,
                0,
                vec![
                    Param::Register(9),
                    Param::Register(6),
                    Param::RegisterORImm5,
                ],
            ),
        );

        instr_set.insert(
            String::from("BR"),
            InstrDef::new(OP::BR, 0, vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRn"),
            InstrDef::new(OP::BR, flag_set_mask(11), vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRz"),
            InstrDef::new(OP::BR, flag_set_mask(10), vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRp"),
            InstrDef::new(OP::BR, flag_set_mask(9), vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRnz"),
            InstrDef::new(
                OP::BR,
                flag_set_mask(11) + flag_set_mask(10),
                vec![Param::Label],
            ),
        );

        instr_set.insert(
            String::from("BRnp"),
            InstrDef::new(
                OP::BR,
                flag_set_mask(11) + flag_set_mask(9),
                vec![Param::Label],
            ),
        );

        instr_set.insert(
            String::from("BRzp"),
            InstrDef::new(
                OP::BR,
                flag_set_mask(10) + flag_set_mask(9),
                vec![Param::Label],
            ),
        );

        instr_set.insert(
            String::from("BRnzp"),
            InstrDef::new(
                OP::BR,
                flag_set_mask(10) + flag_set_mask(9) + flag_set_mask(11),
                vec![Param::Label],
            ),
        );

        instr_set.insert(
            String::from("JMP"),
            InstrDef::new(OP::JMP, 0, vec![Param::Label]),
        );

        let nzp = flag_set_mask(10) + flag_set_mask(9) + flag_set_mask(11);

        instr_set.insert(
            String::from("RET"),
            InstrDef::new(OP::BR, 0b0000_000_111_000000, vec![]),
        );
        instr_set.insert(
            String::from("JSR"),
            InstrDef::new(OP::JSR, binary_utils::flag_set_mask(11), vec![Param::Label]),
        );

        instr_set.insert(String::from("JSSR"), InstrDef::new(OP::RES, 0, Vec::new()));

        instr_set.insert(
            String::from("LD"),
            InstrDef::new(OP::LD, 0, vec![Param::Register(9), Param::Label]),
        );

        instr_set.insert(
            String::from("LDI"),
            InstrDef::new(OP::LDI, 0, vec![Param::Register(9), Param::Label]),
        );

        instr_set.insert(
            String::from("LDR"),
            InstrDef::new(
                OP::LDR,
                0,
                vec![Param::Register(9), Param::Register(6), Param::Bits(6)],
            ),
        );

        instr_set.insert(
            String::from("LEA"),
            InstrDef::new(OP::LEA, 0, vec![Param::Register(9), Param::Label]),
        );

        instr_set.insert(
            String::from("NOT"),
            InstrDef::new(
                OP::NOT,
                0b11_1111,
                vec![Param::Register(9), Param::Register(6)],
            ),
        );

        instr_set.insert(
            String::from("RET"),
            InstrDef::new(OP::JMP, 0b111 << 6 /*set register to R7 */, vec![]),
        );
        instr_set.insert(String::from("RTI"), InstrDef::new(OP::RTI, 0, vec![]));

        instr_set.insert(
            String::from("ST"),
            InstrDef::new(OP::ST, 0, vec![Param::Register(9), Param::Label]),
        );
        instr_set.insert(
            String::from("STI"),
            InstrDef::new(OP::STI, 0, vec![Param::Register(9), Param::Label]),
        );
        instr_set.insert(
            String::from("STR"),
            InstrDef::new(
                OP::STR,
                0,
                vec![Param::Register(9), Param::Register(6), Param::Label6bit],
            ),
        );

        instr_set.insert(
            String::from("TRAP"),
            InstrDef::new(OP::TRAP, 0, vec![Param::Bits(8)]),
        );
        instr_set.insert(String::from("HALT"), InstrDef::new(OP::RES, 0, vec![]));

        instr_set
    }
}

#[derive(Debug)]
pub enum Param {
    Bits(u16),
    Register(u16), /*Lower bit [val -> val+2] */
    Label,
    Label6bit,
    RegisterORImm5,
    Imm5,
}
