use crate::binary_utils;
use crate::binary_utils::flag_set_mask;
use crate::binary_utils::instructions;
use core::panic;
use std::mem::Discriminant;
use crate::error;
use crate::virtual_machine;
use io::BufRead;
use std::fs::read;
use std::fs::File;
use std::io;
use std::io::BufReader;

#[derive(Debug)]
pub struct Symbol {
    name: String,
    offset_from_origin: u16,
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
enum Sign {
    PLUS = 1,
    MINUS = -1,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
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

#[derive(Debug)]
#[derive(Clone)]
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
    Alphabetic_LblRegOrInstr,
}

impl PartialEq for Token{
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

    pub fn is(&self, other: &Self) -> bool{
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    pub fn is_valid_arg(&self, param: &Param)-> bool{
        match self{
            Self::Comma|Self::Directive(_)|Self::Instruction(_)|Self::StringLiteral(_) => return false,
            _ => {},
        };

        match param{
            Param::Label => {
                return self.is(&Token::Label(String::new()))
            },

            Param::Register => {
                return self.is(&Token::Register(0))
            },

            Param::Bits(bits) => {
                match self{
                    Self::DecimalLiteral(number)|Self::HexLiteral(number) => {
                        return number.bits <= *bits-1;
                    },

                    _ => return false,
                }
            },

            Param::Imm5 => panic!("INTERNAL ERROR: \tUnexpected instr to be defined as having imm5."),

            Param::RegisterORImm5 => {
                return self.is(&Token::Register(0)) || 
                    match self { Self::DecimalLiteral(number)|Self::HexLiteral(number) => {number.bits <= 5-1}, _ => false};
            }
        }

        /*match param{
            Param::Label => {
                if self.is(&Token::Label(String::new())){
                    return Param::Label;
                }
            },
            Param::Register => self.is(&Self::Register(0)),
            Param::RegisterORImm5 => self.is(&Self.)
        }*/
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
                    if c == ' ' || c == '\n' {
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
                        current_token = Some(Self::Alphabetic_LblRegOrInstr);

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
                        Self::Alphabetic_LblRegOrInstr => {
                            //Register
                            if c.is_alphanumeric() {
                                current_token_text.push(c);
                                continue;
                            }

                            //Terminators
                            if c == ',' || c == ' ' || c == '\n' {
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
                            } else if c == ',' || c == ' ' || c == '\n' {
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
                            } else if c == ',' || c == ' ' || c == '\n' {
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
                                if value > 2u32.pow(15) {
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
                                if c == ',' || c == ' ' || c == '\n' {
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

pub struct Assembler {
    file_path: String,
    raw_lines: Vec<String>,
    processed_lines: Vec<SourceLine>,
    tokenized_lines: Vec<(Vec<Token>, u16)>,
    symbol_table: SymbolTable,
    instruction_set: HashMap<String, InstrDef>,
    orig: u16,
    end: u16,
}

impl Assembler {
    pub fn new(path: &str) -> Self {
        Self {
            file_path: path.to_string(),
            raw_lines: Vec::new(),
            processed_lines: Vec::new(),
            tokenized_lines: Vec::new(),
            symbol_table: Vec::new(),
            instruction_set: Parser::define_instruction_set(),
            orig: 0,
            end: 0,
        }
    }

    pub fn load(&mut self) {
        
        let file_open_result = File::open(self.file_path.as_str());

        let mut file = match file_open_result {
            Ok(f) => f,
            Err(e) => {
                dbg!(e);
                panic!("{:?}", error::FileLoadError::FsOpenFailed);
            }
        };

        let mut buf = BufReader::new(file);

        let file_read_result: Vec<String> = buf
            .lines()
            .map(|l| l.expect("Failure to read line"))
            .collect();

        self.raw_lines = file_read_result;
        self.processed_lines = self.omit_comments();
        for ln in &self.processed_lines {
            println!("{:03}\t{}", ln.actual_line, ln.text);
        }
        self.symbol_table = Self::build_symbol_table(&self.processed_lines);
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
                });
            });
        }

        table
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
                    return;
                }
            };

            println!("{:02}     {:?}", ln.actual_line, token_stream);
            self.tokenized_lines.push((token_stream, ln.number));
        }
    }

    pub fn parse(&mut self){
        let instruction_set = Parser::define_instruction_set();

    }

    pub fn parse_origin_and_end(&mut self) -> Result<(u16, u16), String>{
        let mut found_orig = false;
        let mut expecting_origin_value_next = false;

        let mut found_end = false;
        

        //let expected_origin_tk = &self.tokenized_lines.first().expect("Expected origin.").0.first().expect("Expected token");
       

        for ln in &self.tokenized_lines{
            let token_stream = &ln.0;
            
            println!("{:03}\t{:?}", ln.1, ln.0);

            for token in token_stream{
                match token{
                    Token::Directive(dir) => {
                        if dir != "ORIG"{
                            if !found_orig{
                                return Err(String::from("Expected .ORIG directive. Found directive '.{dir}' instead."));
                            }/*else if dir != "END" && !found_end{
                                return Err(format!("Expected .END directive. Found directive '.{dir}' instead."));
                            }*/else if dir == "END"{

                                if found_end{
                                    return Err(format!(".END already defined ({:x}).", self.end))
                                }

                                self.end = ln.1 + self.orig;
                                found_end = true;
                            }
                        }else{
                            if found_orig{
                                return Err(format!(".ORIG aleady defined ({}).", self.orig));
                            }else{
                                expecting_origin_value_next = true;
                            }
                        }
                    },

                    Token::DecimalLiteral(val)|Token::HexLiteral(val) => {
                        if !expecting_origin_value_next && !found_orig{
                            return Err(format!("Not expecting decimal literal."));
                        }else{
                            if !found_orig {
                                match val.sign {
                                    Sign::MINUS =>  {return Err(format!(".ORIG must be set to a positive value."))},
                                    _ => {},
                                }

                                self.orig = val.value;
                                found_orig = true;
                                expecting_origin_value_next = false;
                            }
                        }
                    },


                    other => {} /*return Err(if !expecting_origin_value_next {format!("Expected .orig directive. Found {:?} ", other)} else {format!("Expecting number literal.")})*/,
                }
            }

        } 

        if !found_orig{
            return Err(format!("Unable to find .ORIG"));
        }

        if !found_end{
            return Err(format!("Unable to find .END"));
        }

        Ok((self.orig, self.end))
    }

    fn parse_directive(&mut self){
        for ln in &self.tokenized_lines{
            let token_stream = &ln.0;

            
        }  
    }

    pub fn parse_instructions(&mut self){
        

        println!("\n Removing leading labels.");
        for (line, line_offset) in &self.tokenized_lines{
            let line = match line.strip_prefix(&[Token::Label(format!(""))]){
                Some(without_label) => without_label.clone(),
                None => line,
            };     
            //println!("{line_offset} \t {line:?}");
            self.parse_single_instr(line.to_vec(), *line_offset);
            
        }

       
    }

    fn parse_single_instr(&self, line: Vec<Token>, line_offset: u16){
        let mut target_instruction: &InstrDef = &InstrDef::new(OP::RES, 0, vec![]);

        if line.starts_with(&[Token::Instruction(format!(""))]){
            //Find which instruction it is; 
            match line.first().unwrap(){
                Token::Instruction(instr) => {
                    println!("{:03} Searching for instruction '{instr}'.", line_offset);
                    target_instruction = &self.instruction_set[instr];//.expect("Undefined instruction {instr}");
                    
                },
                // /Token::Directive(dir) => {
                //     println!("Ignoring directive .{dir} (line {line_offset})");
                // },
                _ => panic!("Expected instr, fatal error."),
            }
            
        }else{ 
            println!("Ignoring line {line:?}");
            return;
        }

        //Get line's paramaters 
        let mut args: Vec<Token> = vec![];
        let mut arg_index = 0;

        let mut previous: Vec<Token> = Vec::new();

        for i in 0..line.len(){
            if i == 0{
                //Ignore instruction
                continue;
            }

           // println!("Token: '{:?}', i = {i}, arg_index = {arg_index}", line[i]);
           

            if arg_index == target_instruction.params.len(){
                panic!("Expected {} operands, found {}. \t[{:?}]", target_instruction.params.len(), arg_index, line);
            }

            if (i % 2) == 0 && !line[i].is(&Token::Comma){
                panic!("Expecting comma between params. {:?}", line);
            }

            if line[i].is(&Token::Comma){
                continue;
            }

            if i%2 != 0 && !line[i].is_valid_arg(&target_instruction.params[arg_index]){
                panic!("Expected {:?} for instruction '{}', found token '{:?}'", target_instruction.params[arg_index], target_instruction.opcode, line[i]);
            }

            //previous.push(line[i].clone());

            args.push(line[i].clone());
            arg_index += 1;

        }

        println!("{line_offset}\t{args:?}\t{target_instruction:?}");

    }
}
    /*pub fn find_origin_and_end(&mut self){
        let first_nc_line = self.processed_lines.first().unwrap().text.trim_start();

        if(first_nc_line.chars() != '.'){

        }

        for c in first_nc_line.chars(){

        }
    }*/


pub type SymbolTable = Vec<Symbol>;

/*
pub fn read_asm_file(path: &str) -> Result<SourceLines, error::FileLoadError> {
    let file_open_result = File::open(path);

    let mut file = match file_open_result {
        Ok(f) => f,
        Err(e) => {
            dbg!(e);
            return Err(error::FileLoadError::FsOpenFailed);
        }
    };

    let mut buf = BufReader::new(file);

    let file_read_result: Vec<String> = buf
        .lines()
        .map(|l| l.expect("Failure to read line"))
        .collect();

    // match file_read_result{
    //     Ok(_) => {},
    //     Err(_) => {
    //         return Err(error::FileLoadError::FsReadFailed);
    //     }
    // };

    Ok(omit_comments(file_read_result))
}*/

/*  pub fn omit_comments(lines: Vec<String>) -> Vec<SourceLine> {
//     let mut result: Vec<SourceLine> = Vec::new();


//     let mut non_comment_line_found = false;
//     let mut skip_count: u16= 0;

//     for i in 0..lines.len() {
//         if lines[i].trim_start().starts_with(";") && !non_comment_line_found {
//             skip_count += 1;
//             continue;
//         }

//         non_comment_line_found = true;

//         let mut line = String::new();
//         for char in lines[i].chars(){
//             if char == ';'{
//                 break;
//             }

//             line.push(char);
//         }
//         let n: u16 = i.try_into().unwrap();

//         result.push(SourceLine::new(lines[i].as_str(), n - skip_count, n));
//     }
//     result
// }*/




fn parse_line_for_symbol(line: &SourceLine) -> Option<String> {
    //First remove comments
    let mut contains_comment = false;

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
        "AND", "ADD", "NOT", "BR", "BRz", "BRp", "BRn", "BRzn", "BRznp", "BRnp", "BRzp", "LD",
        "LDI", "LDR", "ST", "STR", "STI", "TRAP", "JMP", "RET", "JSR", "JSRR", "LEA", "HALT",
    ]
    .contains(&s)
}

/* // pub mod token{
//     use std::thread::current;

//     use crate::error;

    pub enum Category {
        DecimalLiteral, HashSymbol,
        HexLiteral, LowerCaseX,
        Label,
        Directive,
        Instruction,
        Comma,
        OpenQuote,
        EndQuote,
        Register,
        Invalid,
        Whitespace,
        Empty,
    }

//     pub enum CharClassification{
//         DecimalLiteral, HexLiteral,
//         HashSymbol, LowerCaseX,

//         Dot,

//         CapitalR,

//         Whitespace,

//         Other,
//     }


//     pub struct Token{
//         text: String,
//         category: Category,
//     }

//     impl Token{
//         fn new() -> Self{
//             Self {
//                 text: String::new(),
//                 category: Category::Empty,
//             }
//         }
//     }

//     pub fn tokenize_line(line: String) -> Result<Vec<Token>, error::AssemblerError>{
//         let mut line = line.trim().chars();
//         let mut tokenized: Vec<Token> = Vec::new();

//         let mut current_token_type: Option<Token>;

//         line.map(|c: &str| {

//         });

//         Ok(tokenized)
//     }



//     pub fn classify_character(c: &str)-> CharClassification{
//         let c =String::from(c.clone());
//         let mut classification = CharClassification::Other;

//         if c.len() != 1{
//             panic!("Cannot classify char '{}'; len = ", c.len());
//         }else  if !c.is_ascii(){
//             panic!("Cannot process non-ascii characters.");
//         }


//         return classification;
//     }
// } */
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
            opcode: opcode as u16,
            flags_word,
            params,
        }
    }
}

struct Parser {
    lines: Vec<String>,
    symbolTable: SymbolTable,
    instruction_set: HashMap<String, InstrDef>,
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
                vec![Param::Register, Param::Register, Param::RegisterORImm5],
            ),
        );
        instr_set.insert(
            String::from("AND"),
            InstrDef::new(
                OP::AND,
                0,
                vec![Param::Register, Param::Register, Param::RegisterORImm5],
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
            InstrDef::new(OP::BR, flag_set_mask(11) + flag_set_mask(10), vec![Param::Label]),
        );


        instr_set.insert(
            String::from("BRnp"),
            InstrDef::new(OP::BR, flag_set_mask(11) + flag_set_mask(9), vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRzp"),
            InstrDef::new(OP::BR, flag_set_mask(10) + flag_set_mask(9), vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRnzp"),
            InstrDef::new(OP::BR,  flag_set_mask(10) + flag_set_mask(9) + flag_set_mask(11), vec![Param::Label]),
        );

        instr_set.insert(
            String::from("JMP"),
            InstrDef::new(OP::JMP, 0, vec![Param::Label]),
        );
        instr_set.insert(
            String::from("JSR"),
            InstrDef::new(OP::JSR, binary_utils::flag_set_mask(11), vec![Param::Label]),
        );

        instr_set.insert(String::from("JSSR"), InstrDef::new(OP::RES, 0, Vec::new()));

        instr_set.insert(
            String::from("LD"),
            InstrDef::new(OP::LD, 0, vec![Param::Register, Param::Label]),
        );

        instr_set.insert(
            String::from("LDI"),
            InstrDef::new(OP::LDI, 0, vec![Param::Register, Param::Label]),
        );

        instr_set.insert(
            String::from("LDR"),
            InstrDef::new(
                OP::LDR,
                0,
                vec![Param::Register, Param::Register, Param::Bits(6)],
            ),
        );

        instr_set.insert(String::from("LEA"), InstrDef::new(OP::LEA, 0, vec![Param::Register, Param::Label]));

        instr_set.insert(String::from("NOT"), InstrDef::new(OP::NOT, 0, vec![Param::Register, Param::Register]));

        instr_set.insert(String::from("RET"), InstrDef::new(OP::JMP, 0b111<<6 /*set register to R7 */, vec![]));
        instr_set.insert(String::from("RTI"), InstrDef::new(OP::RTI, 0, vec![]));

        instr_set.insert(String::from("ST"), InstrDef::new(OP::ST, 0, vec![Param::Register, Param::Label]));
        instr_set.insert(String::from("STI"), InstrDef::new(OP::STI, 0, vec![Param::Register, Param::Label]));
        instr_set.insert(
            String::from("STR"),
            InstrDef::new(OP::STR, 0, vec![Param::Register, Param::Register, Param::Bits(6)]),
        );

        instr_set.insert(String::from("TRAP"), InstrDef::new(OP::TRAP, 0, vec![Param::Bits(8)]));
        instr_set.insert(String::from("HALT"), InstrDef::new(OP::RES, 0, vec![]));

        instr_set
    }

    fn parse_instructions() {}
}

#[derive(Debug)]
enum Param {
    Bits(u16),
    Register,
    Label,
    RegisterORImm5,
    Imm5,
}

/*struct InstructionSyntax{
    operands: Vec<Operand>,
}*/

fn syntax() {}
