use crate::binary_utils;
use crate::binary_utils::instructions;
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

#[derive(Debug)]
enum Sign {
    PLUS = 1,
    MINUS = -1,
}

#[derive(Debug)]
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
pub enum Token {
    DecimalLiteral(NumberLiteral),
    HexLiteral(NumberLiteral),
    Register(u16),
    Label(String),
    Comma,
    Instruction(String),
    Directive(String),
    NullTermString(String),
    Alphabetic_LblRegOrInstr,
}




impl Token {
    pub fn tokenize_str(line: &str) -> Vec<Token>{
        Self::tokenize_line(&SourceLine::new(line, 0, 0))
    }

    pub fn is_directive(name: &str) -> bool{
        vec!["BLKW", "FILL", "ORIG", "END", "STRINGZ"].contains(&name)
    }

    pub fn tokenize_line(line: &SourceLine) -> Vec<Token> {

      

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
                    if c == ' ' || c == '\n'{
                        continue;
                    }

                    if c == ',' {
                        token_stream.push(Self::Comma);
                        continue;
                    }

                    if c == '"' {
                        current_token = Some(Self::NullTermString(String::new()));
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

                    panic!("Invalid character '{}' ", c)
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
                            if c == ',' || c == ' ' || index+1 >= line.text.trim().len() {
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
                                    }else{
                                        panic!("Invalid register 'R{}'. Valid registers: R0, R1, ... R7.", register_no);
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
                                && (c != '-') && c != '\n'
                            {
                                panic!("Invalid decimal literal.");
                            } else {
                                if c == '-' {current_token_text.push(c)};
                               
                            }
                            
                            if c.is_ascii_digit() {
                                current_token_text.push(c);
                            }else
                                if c == ',' || c == ' ' || c == '\n' || index +1 >= line.text.trim().len() {

                                    if index+1 == line.text.len(){
                                        current_token_text.push(c);
                                    }

                                    let mut interpretation: NumberLiteral = NumberLiteral::new();

                                    if current_token_text.starts_with("-"){ 
                                        interpretation.sign = Sign::MINUS;
                                    }
                                    current_token_text = current_token_text.trim_start_matches('-').to_string();
                                    //println!("'{}'", current_token_text);
                                   // break;
                                    let value: u32 = current_token_text.parse().unwrap();
                                    if value > 2u32.pow(15){
                                        panic!("Decimal literal {} is out of range.", value);
                                    }



                                    interpretation.value = value.try_into().expect("Unable to convert decimal literal u32 to u16");

                                    interpretation.bits = binary_utils::bits_required_for_number(interpretation.value);

                                    token_stream.push(Self::DecimalLiteral(interpretation));
                                    current_token_text.clear();
                                    
                                    if c == ',' {
                                        token_stream.push(Self::Comma);
                                    }
                                }
                            
                        },

                        Self::HexLiteral(_) => {
                            if !c.is_ascii_hexdigit() 
                                && current_token_text.len() != 0
                                && (c != '-') && c != '\n'
                            {
                                panic!("Invalid decimal literal.");
                            } else {
                                if c == '-' {current_token_text.push(c)};
                               
                            }
                            
                            if c.is_ascii_hexdigit() {
                                current_token_text.push(c);
                            }else
                                if c == ',' || c == ' ' || c == '\n' {

                                    if index+1 == line.text.len(){
                                        current_token_text.push(c);
                                    }

                                    let mut interpretation: NumberLiteral = NumberLiteral::new();

                                    if current_token_text.starts_with("-"){ 
                                        interpretation.sign = Sign::MINUS;
                                    }
                                    current_token_text = current_token_text.trim_start_matches('-').to_string();
                                    //println!("'{}'", current_token_text);
                                   // break;
                                    let value: u32 = u32::from_str_radix(&current_token_text, 16).unwrap();
                                    if value > 2u32.pow(15){
                                        panic!("Hexadecimal literal {:0x} is out of range.", value);
                                    }



                                    interpretation.value = value.try_into().expect("Unable to convert hexadecimal literal u32 to u16");

                                    interpretation.bits = binary_utils::bits_required_for_number(interpretation.value);

                                    token_stream.push(Self::DecimalLiteral(interpretation));
                                    current_token_text.clear();
                                    
                                    if c == ',' {
                                        token_stream.push(Self::Comma);
                                    }
                                }
                            
                        },

                        Self::Directive(_) => {
                            if !c.is_ascii_alphabetic(){
                                
                                if c == ',' || c == ' ' || c == '\n'{
                                    if !Token::is_directive(&current_token_text){
                                        panic!("'.{}' is not a valid directive.", current_token_text);
                                    }else{
                                        token_stream.push(Self::Directive(current_token_text.clone()));
                                    }

                                    if c == ',' {
                                        token_stream.push(Self::Comma);
                                    }

                                    current_token = None;
                                    current_token_text.clear();
                                }else{
                                    panic!("Invalid directive.");
                                }
                            }else{
                                current_token_text.push(c);
                            }

                        },
                        _ => /*panic!()*/{},
                    }
                }
            }
        }
        //println!("{}", current_token_text);


        token_stream
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

type SourceLines = Vec<SourceLine>;
pub struct Assembler {
    file_path: String,
    raw_lines: Vec<String>,
    processed_lines: Vec<SourceLine>,
    tokenized_lines: Vec<Vec<Token>>,
    symbol_table: SymbolTable,
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
                    break;
                }

                line.push(char);
            }
            let n: u16 = i.try_into().unwrap();

            result.push(SourceLine::new(
                self.raw_lines[i].as_str(),
                n - skip_count,
                n + 1,
            ));
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

    pub fn tokenize(&mut self){
        for ln in &self.processed_lines{
            let token = Token::tokenize_line(ln);
            println!("{:02}     {:?}", ln.actual_line, token);
            self.tokenized_lines.push(token);
        }
    }


    /*pub fn find_origin_and_end(&mut self){
        let first_nc_line = self.processed_lines.first().unwrap().text.trim_start();

        if(first_nc_line.chars() != '.'){

        }

        for c in first_nc_line.chars(){

        }
    }*/
}

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

fn find_program_origin() {}

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

// fn parse_line_for_origin(line: &str) -> Option<String> {
//     //First remove comments

// }

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

pub mod assembler {
    use std::collections::HashMap;

    use crate::binary_utils::instructions;

    // pub enum Token {
    //     DecimalLiteral(u16),
    //     HashSymbol,
    //     HexLiteral(u16),
    //     LowerCaseX,
    //     Label(String),
    //     Directive,
    //     Instruction,
    //     Comma,
    //     Dot,
    //     Text,
    //     Register(u16),
    //     Invalid,
    //     Whitespace,
    //     Empty,
    // }

    // struct Tokenizer {
    //     lines: Vec<String>,
    // }

    // impl Tokenizer {
    //     fn new(lines: Vec<String>) -> Self {
    //         Tokenizer {
    //             lines: lines.clone(),
    //         }
    //     }
    // }

    struct Parser {
        lines: Vec<String>,
        symbolTable: super::SymbolTable,
        instruction_set: HashMap<String, Vec<Param>>,
    }

    impl Parser {

       
        fn define_instruction_set() -> HashMap<String, Vec<Param>> {
            let mut instr_set = HashMap::new();
            instr_set.insert(
                String::from("ADD"),
                vec![Param::Register, Param::Register, Param::RegisterORImm5],
            );
            instr_set.insert(
                String::from("AND"),
                vec![Param::Register, Param::Register, Param::RegisterORImm5],
            );

            instr_set.insert(String::from("BR"), vec![Param::Label]);

            instr_set.insert(String::from("JMP"), vec![Param::Register]);
            instr_set.insert(String::from("JSR"), vec![Param::Label]);

            instr_set.insert(String::from("JSSR"), vec![Param::Register]);

            instr_set.insert(String::from("LD"), vec![Param::Register, Param::Label]);
            instr_set.insert(String::from("LDI"), vec![Param::Register, Param::Label]);

            instr_set.insert(
                String::from("LDR"),
                vec![Param::Register, Param::Register, Param::Bits(6)],
            );
            instr_set.insert(String::from("LEA"), vec![Param::Register, Param::Label]);

            instr_set.insert(String::from("NOT"), vec![Param::Register, Param::Register]);

            instr_set.insert(String::from("RET"), vec![]);
            instr_set.insert(String::from("RTI"), vec![]);

            instr_set.insert(String::from("ST"), vec![Param::Register, Param::Label]);
            instr_set.insert(String::from("STI"), vec![Param::Register, Param::Label]);
            instr_set.insert(
                String::from("STR"),
                vec![Param::Register, Param::Register, Param::Bits(6)],
            );

            instr_set.insert(String::from("TRAP"), vec![Param::Bits(8)]);

            instr_set
        }

        fn parse_instructions(){
        }
    }

    enum Param {
        Bits(u16),
        Register,
        Label,
        RegisterORImm5,
    }

    /*struct InstructionSyntax{
        operands: Vec<Operand>,
    }*/

    fn syntax() {}
}
