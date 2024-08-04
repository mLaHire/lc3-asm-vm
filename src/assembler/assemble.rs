use crate::binary_utils::*;
use crate::error::AsmblrErr;
use crate::tokenizer::*;
use crate::error::FileLoadError;
use crate::virtual_machine;
use core::panic;
use io::BufRead;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::thread;
use std::time;

#[derive(Clone, Debug)]
pub struct Symbol {
    name: String,
    rel_addr: u16,
    abs_addr: u16,
    src_ln_number: u16,
    size_in_words: u16,
    is_external: bool,
}





pub struct TrapInstruction {
    instructions: Vec<u16>,
    origin: u16,
    trap_vector: u16,
    memory_writes: Vec<(u16, u16)>,
}

impl TrapInstruction {
    pub fn new(filename: &str, trap_vector: u16) -> Self {
        let mut asm = Assembler::new(format!(".\\src\\asm_files\\trap\\{}.asm", filename).as_str());
        asm.load();
        match asm.tokenize() {
            Ok(_) => (),
            Err(errors) => {
                AsmblrErr::display(
                    &format!(".\\src\\asm_files\\trap\\{}.asm", filename),
                    &asm.raw_lines,
                    &errors,
                );
                panic!();
            }
        }
        match asm.parse_origin_and_end() {
            Err(errors) => {
                AsmblrErr::display(
                    &format!(".\\src\\asm_files\\trap\\{}.asm", filename),
                    &asm.raw_lines,
                    &errors,
                );
                panic!();
            }
            Ok(r) => println!("TRAP Program\t.ORIG {:x}\t.END{:x}", r.0, r.1),
        };

        match asm.load_symbols() {
            Err(errors) => {
                AsmblrErr::display(
                    &format!(".\\src\\asm_files\\trap\\{}.asm", filename),
                    &asm.raw_lines,
                    &errors,
                );
                panic!();
            }

            Ok(_) => {},
           // Ok(r) => println!("TRAP Program\t.ORIG {:x}\t.END{:x}", r.0, r.1),
        };
        asm.adjust_symbols();
        //asm.trim_lines();
        let memory_writes = match asm.parse_directives_to_list() {
            Ok(writes) => writes,
            Err(errors) => {
                AsmblrErr::display(
                    &format!(".\\src\\asm_files\\trap\\{}.asm", filename),
                    &asm.raw_lines,
                    &errors,
                );
                panic!();
            }
        };
        let instructions = match asm.parse_instructions() {
            Ok(writes) => writes,
            Err(errors) => {
                AsmblrErr::display(
                    &format!(".\\src\\asm_files\\trap\\{}.asm", filename),
                    &asm.raw_lines,
                    &errors,
                );
                panic!();
            }
        };
        TrapInstruction {
            memory_writes,
            instructions,
            trap_vector,
            origin: asm.orig,
        }
    }
}

#[derive(Clone)]
pub struct TokenizedLine {
    pub rel_addr: u16,
    pub src_ln_number: u16,
    pub tokens: Vec<Token>,
}

pub struct MemoryWrite {
    pub rel_addr: u16,
    pub value: u16,
}

pub struct ExecutableImage {
    pub name: String,
    pub origin: u16,
    pub instructions: Vec<MemoryWrite>,
    pub data: Vec<MemoryWrite>,
    pub symbol_table: Vec<Symbol>,
}

impl ExecutableImage {
    pub fn new(name: String) -> Self {
        ExecutableImage {
            name: name.clone(),
            origin: 0,
            instructions: Vec::new(),
            data: Vec::new(),
            symbol_table: Vec::new(),
        }
    }
}

pub struct Assembler {
    file_path: String,
    pub raw_lines: Vec<String>,
    processed_lines: Vec<SourceLine>,
    //tokenized_lines: Vec<(Vec<Token>, u16)>,
    tokenized_lines: Vec<TokenizedLine>,
    symbol_table: SymbolTable,
    instruction_set: HashMap<String, InstrDef>,
    pub vm: virtual_machine::VirtualMachine,
    pub orig: u16,
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
            instruction_set: InstructionSet::define_instruction_set(),
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
                panic!("{:?}", FileLoadError::FsOpenFailed);
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

    pub fn assemble(&mut self) -> Result<ExecutableImage, Vec<AsmblrErr>> {
        let mut errors = Vec::new();
        let mut img = ExecutableImage::new(self.file_path.clone());

        match self.tokenize() {
            Ok(_) => (),
            Err(e) => errors = [errors, e].concat(),
        };
        match self.parse_origin_and_end() {
            Err(e) => {
                errors = [errors, e].concat();
            }
            Ok(r) => {
                println!("[ASM] Program\t.ORIG {:x}\t.END{:x}", r.0, r.1);
                img.origin = r.0;
            }
        }
        match self.load_symbols() {
            Ok(_) => (),
            Err(e) => {
                errors = [errors, e].concat();
            }
        }

        match self.parse_directives_to_list() {
            Ok(list) => {
                for (addr, value) in list {
                    img.data.push(MemoryWrite {
                        rel_addr: addr - self.orig,
                        value,
                    })
                }
            }
            Err(e) => {
                errors = [errors, e].concat();
            }
        }
        self.adjust_symbols();

        match self.parse_instructions() {
            Ok(instructions) => {
                for (index, word) in instructions.into_iter().enumerate() {
                    img.instructions.push(MemoryWrite {
                        rel_addr: index as u16,
                        value: word,
                    })
                }
            }
            Err(e) => {
                errors = [errors, e].concat();
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        img.symbol_table = (self.symbol_table).clone();

        Ok(img)
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
                //skip_count += 1; //Attempt to fix bug with empty lines skewing symbol gen
                continue;
            }

            result.push(SourceLine::new(&line.clone(), n - skip_count, n + 1));
        }
        result
    }

    pub fn load_symbols(&mut self) -> Result<(), Vec<AsmblrErr>> {
        let mut errors = Vec::new();
        self.omit_empty_lines();
        for tk_ln in &self.tokenized_lines {
            let token_stream = &tk_ln.tokens;
            let relative_address = tk_ln.rel_addr;
            if let Token::Label(symbol) = token_stream.first().unwrap() {
                if self
                    .symbol_table
                    .iter()
                    .find(|&sym| sym.name.eq_ignore_ascii_case(symbol))
                    .is_some()
                {
                    errors.push(AsmblrErr::new(
                        tk_ln.src_ln_number,
                        format!("Label '{}' is already defined.", symbol),
                    ));
                    let initial_def = self
                        .symbol_table
                        .iter()
                        .find(|&sym| sym.name.eq_ignore_ascii_case(symbol))
                        .unwrap();
                    errors.push(AsmblrErr::new(
                        initial_def.src_ln_number,
                        format!("Label {} is defined again later.", symbol),
                    ))
                }
                self.symbol_table.push(Symbol {
                    name: symbol.to_string().clone(),
                    rel_addr: relative_address,
                    abs_addr: 0,
                    src_ln_number: tk_ln.src_ln_number,
                    size_in_words: 1,
                    is_external: false,
                });
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        println!("Symbol table: {:#?}", self.symbol_table);
        Ok(())
    }

    pub fn tokenize(&mut self) -> Result<Vec<TokenizedLine>, Vec<AsmblrErr>> {
        let mut tokenized_lines = Vec::new();
        let mut errors = Vec::new();
        for ln in &self.processed_lines {
            let token_stream = match Token::tokenize_line(ln) {
                Ok(tk) => tk,
                Err(e) => {
                    errors.push(AsmblrErr::new(
                        ln.actual_line,
                        e,
                        /*  format!(
                        "\nSyntax error ('{}' (line {})):\n\n{:02}\t\t'{}'\n\n\t\t{}",
                        self.file_path, ln.actual_line, ln.actual_line, ln.text, e*/
                    ));
                    continue;
                }
            };

            println!("{:02}     {:?}", ln.actual_line, token_stream);
            tokenized_lines.push(TokenizedLine {
                rel_addr: ln.number,
                src_ln_number: ln.actual_line,
                tokens: token_stream,
            });
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        self.tokenized_lines = tokenized_lines.clone();
        Ok(tokenized_lines)
    }

    pub fn omit_empty_lines(&mut self) {
        //!(tk.0.is_empty() || (tk.0.starts_with(&[Token::Directive(String::new())]))
        let filtered = self
            .tokenized_lines
            .clone()
            .into_iter()
            .filter(|tk_ln| {
                if tk_ln.tokens.is_empty() {
                    false
                } else if let Token::Directive(dir) = tk_ln.tokens.first().unwrap() {
                    let mut dir = dir.clone();
                    dir.make_ascii_uppercase();
                    match dir.as_str() {
                        "ORIG" | "END" => false,
                        _ => true,
                    }
                } else {
                    true
                }
            })
            .enumerate();

        self.tokenized_lines.clear();
        for (index, line) in filtered {
            self.tokenized_lines.push(TokenizedLine {
                rel_addr: index as u16,
                src_ln_number: line.src_ln_number,
                tokens: line.tokens,
            });
        }
    }

    // pub fn parse(&mut self) {
    //     let instruction_set = Parser::define_instruction_set();
    // }

    pub fn parse_origin_and_end(&mut self) -> Result<(u16, u16), Vec<AsmblrErr>> {
        let mut found_orig = false;
        let mut expecting_origin_value_next = false;

        let mut found_end = false;

        let mut errors = Vec::new();

        //let expected_origin_tk = &self.tokenized_lines.first().expect("Expected origin.").0.first().expect("Expected token");

        for ln in &self.tokenized_lines {
            let token_stream = &ln.tokens;

            println!("{:03}\t{:?}", ln.src_ln_number, token_stream);

            for token in token_stream {
                match token {
                    Token::Directive(dir) => {
                        if dir != "ORIG" {
                            if !found_orig {
                                errors.push(AsmblrErr::new(
                                    ln.src_ln_number,
                                    format!(
                                    "Expected .ORIG directive. Found directive '.{dir}' instead.",
                                ),
                                ));
                            }
                            /*else if dir != "END" && !found_end{
                                return Err(format!("Expected .END directive. Found directive '.{dir}' instead."));
                            }*/
                            else if dir == "END" {
                                if found_end {
                                    errors.push(AsmblrErr::new(
                                        ln.src_ln_number,
                                        format!(".END already defined ({:x}).", self.end),
                                    ));
                                }

                                self.end = ln.rel_addr + self.orig;
                                found_end = true;
                            }
                        } else {
                            if found_orig {
                                errors.push(AsmblrErr::new(
                                    ln.src_ln_number,
                                    format!(".ORIG aleady defined ({}).", self.orig),
                                ));
                            } else {
                                expecting_origin_value_next = true;
                            }
                        }
                    }

                    Token::DecimalLiteral(val)
                    | Token::HexLiteral(val)
                    | Token::BinLiteral(val) => {
                        if !expecting_origin_value_next && !found_orig {
                            errors.push(AsmblrErr::new(
                                ln.src_ln_number,
                                format!("Not expecting decimal literal."),
                            ));
                        } else {
                            if !found_orig {
                                match val.sign {
                                    Sign::MINUS => {
                                        errors.push(AsmblrErr::new(
                                            ln.src_ln_number,
                                            format!(".ORIG must be set to a positive value."),
                                        ));
                                    }
                                    _ => {}
                                }
                                println!("Found origin: {}", { val.value });

                                self.orig = val.value;
                                found_orig = true;
                                expecting_origin_value_next = false;
                            }
                        }
                    }

                    _ => {
                        if expecting_origin_value_next {
                            errors.push(AsmblrErr::new(
                                ln.src_ln_number,
                                format!(
                                    "Found .ORIG directive, but no value is assigned as origin."
                                ),
                            ));
                        }
                    } /*return Err(if !expecting_origin_value_next {format!("Expected .orig directive. Found {:?} ", other)} else {format!("Expecting number literal.")})*/,
                }
            }
            expecting_origin_value_next = false;
        }

        if !found_orig {
            errors.push(AsmblrErr::new(1, format!("Unable to find program .ORIG")));
        }

        if !found_end {
            errors.push(AsmblrErr::new(1, format!("Unable to find program.END")));
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((self.orig, self.end))
    }

    pub fn parse_directives(&mut self) {
        let parsed = match self.parse_directives_to_list() {
            Ok(parsed) => parsed,
            Err(errors) => {
                AsmblrErr::display(&self.file_path, &self.raw_lines, &errors);
                panic!("Unable to parse directives.");
            }
        };
        for (addr, val) in parsed.iter() {
            self.vm.write_memory(*addr, *val);
        }
    }

    /*pub fn __parse_directives(&mut self) {
            todo!("Obsolete");
            println!("[ASM]\tParsing directives...");
            let mut reserved_word_count = 0u16;

            for (line_, line_offset) in &self.tokenized_lines {
                let line = match line_.strip_prefix(&[Token::Label(format!(""))]) {
                    Some(without_label) => {
                        print!("LABEL: {:?}\t", line_.first().unwrap());
                        without_label
                    }
                    None => line_,
                };
                //println!("{line_offset} \t {line:?}");
                let mut line = line.iter().take(2);
                match match line.next() {
                    Some(val) => val,
                    None => continue,
                } {
                    &Token::Directive(ref directive) => {
                        //println!("Found directive {line:?} '{directive}'");
                        if directive != "ORIG"
                            && directive != "END"
                            && *line_offset < (self.end - self.orig + 2)
                        {
                            // println!(
                            //     "WARNING: Expected .END before directive .{directive} line offset = {}",
                            //     *line_offset
                            // );
                        }

                        let unadjusted_offset = *line_offset;

                        let line_offset = line_offset + reserved_word_count;

                        if directive == "FILL" {
                            match line.next() {
                                Some(token) => {
                                    //println!("{:?}", token);
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
                                            if sym.rel_addr == unadjusted_offset {
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
    */
    pub fn adjust_symbols(&mut self) {
        let mut cummulative_offset = 0;
        for symbol in &mut self.symbol_table {
            symbol.rel_addr += cummulative_offset;
            if symbol.size_in_words > 1 {
                cummulative_offset += symbol.size_in_words - 1;
            }
            symbol.abs_addr = self.orig + symbol.rel_addr;
        }
        println!("Adjusted symbol table: {:#?}", self.symbol_table);
    }

    // pub fn resolve_external_symbols(&mut self, symbol_tables: Vec<&Vec<Symbol>>){
        
    // }
    pub fn parse_directives_to_list(&mut self) -> Result<Vec<(u16, u16)>, Vec<AsmblrErr>> {
        let mut memory_writes = Vec::new();
        let mut reserved_word_count = 0u16;
        //let mut skip_count = 0;

        let mut errors = Vec::new();

        for tk_ln in &self.tokenized_lines {
            let line_ = &tk_ln.tokens;
            let line_offset = tk_ln.rel_addr;
            let line = match line_.strip_prefix(&[Token::Label(format!(""))]) {
                Some(without_label) => {
                    println!("LABEL: {:?}\t", line_.first().unwrap());
                    without_label
                }
                None => &line_,
            };
            //println!("{line_offset} \t {line:?}");
            let mut line = line.iter().take(2);
            match match line.next() {
                Some(val) => val,
                None => continue,
            } {
                &Token::Directive(ref directive) => {
                    println!("Found directive .{directive} {:0x}+{line_offset:0x} 0x{:03x} '{directive}'", self.orig, self.orig + line_offset);
                    // if directive != "ORIG"
                    //     && directive != "END"
                    //     && line_offset < (self.end - self.orig/*  - 2*/)
                    // //CAUSED A BUG(!)
                    // {
                    //     // println!(
                    //     //     "WARNING: Expected .END before directive .{directive} line offset = {}",
                    //     //     *line_offset
                    //     // );
                    // }

                    let unadjusted_offset = line_offset;
                    let line_offset = line_offset + reserved_word_count;

                    if directive == "FILL" {
                        match line.next() {
                            Some(token) => {
                                //println!("{:?}", token);
                                if token.is(&Token::HexLiteral(NumberLiteral::new()))
                                    || token.is(&Token::DecimalLiteral(NumberLiteral::new()))
                                {
                                    memory_writes
                                        .push((self.orig + line_offset, token.as_u16(None)));
                                } else {
                                    errors.push(AsmblrErr::new(
                                        tk_ln.src_ln_number,
                                        format!(
                                            "expected a number after .FILL directive, found {:?}",
                                            token
                                        ),
                                    ));
                                }
                            }
                            None => errors.push(AsmblrErr::new(
                                tk_ln.src_ln_number,
                                format!("expected a number after .FILL directive, found nothing"),
                            )),
                        }
                    } else if directive == "STRINGZ" {
                        match line.next() {
                            Some(token) => {
                                // println!("{:?}", token);
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
                                        println!(
                                            "\t'{}' --> 0x{:04x}",
                                            std::str::from_utf8(&[ch]).unwrap(),
                                            self.orig + line_offset + (i as u16)
                                        );
                                    }

                                    //Null term
                                    memory_writes.push((
                                        self.orig + line_offset + (text.bytes().len() as u16),
                                        0,
                                    ));
                                    println!(
                                        "\t'\\0' --> 0x{:04x}\n",
                                        self.orig + line_offset + (text.bytes().len() as u16)
                                    );
                                    reserved_word_count += text.bytes().len() as u16;
                                    for sym in &mut self.symbol_table {
                                        if sym.rel_addr == unadjusted_offset {
                                            sym.size_in_words = 1 + text.bytes().len() as u16;
                                        }
                                    }
                                } else {
                                    errors.push(AsmblrErr::new(
                                        tk_ln.src_ln_number,
                                        format!(
                                            "expected a string literal after .STRINGZ directive, found {:?}",
                                            token
                                        ),
                                    ));
                                }
                            }
                            None => println!("Empty."),
                        }
                    } else if directive == "BLKW" {
                        match line.next() {
                            Some(token) => {
                                //println!("{:?}", token);
                                if token.is(&Token::HexLiteral(NumberLiteral::new()))
                                    || token.is(&Token::DecimalLiteral(NumberLiteral::new()))
                                {
                                    let size_of_block = token.as_u16(None);
                                    if size_of_block > 1 {
                                        reserved_word_count += size_of_block - 1;
                                        for sym in &mut self.symbol_table {
                                            if sym.rel_addr == unadjusted_offset {
                                                sym.size_in_words = size_of_block;
                                            }
                                        }
                                    }

                                    for _ in 0..size_of_block{
                                        memory_writes
                                        .push((self.orig + line_offset, 0));
                                    }
                                   
                                    

                                    println!("Reserving {} words", token.as_u16(None));
                                } else if let Token::Label(text) = token{
                                    match text.trim().parse::<u16>(){
                                        Ok(size_of_block) => {
                                            if size_of_block > 1 {
                                                reserved_word_count += size_of_block - 1;
                                                for sym in &mut self.symbol_table {
                                                    if sym.rel_addr == unadjusted_offset {
                                                        sym.size_in_words = size_of_block;
                                                    }
                                                }
                                                println!("Reserving {} words", text);
                                            }
                                        }
                                        
                                        Err(e) => {
                                            errors.push(AsmblrErr { line_number: tk_ln.src_ln_number, msg: format!("Expected a valid number decimal number after directive .BLKW, found '{text}'")})
                                        }
                                    }
                                }
                            }
                            None => println!("BLKW empty."),
                        }
                    } else if directive == "EXTERNAL"{
                        //skip_count += 1;
                    }
                }

                _ => /*skip_count += 1*/{}
            };
        }
        //memory_writes =  memory_writes.into_iter().map(|w|(w.0-1,w.1)).collect();
        if !errors.is_empty() {
            return Err(errors);
        }

        println!("MEM_WRITES: {:?}", memory_writes);
        Ok(memory_writes)
    }

    pub fn link_then_execute(
        &mut self,
        img: &ExecutableImage,
        trap_instructions: Option<Vec<TrapInstruction>>,
    ) {
        // let mut instructions: Vec<u16> = match self.parse_instructions() {
        //     Ok(instrs) => instrs,
        //     Err(errs) => {
        //         AsmblrErr::display(&self.file_path, &self.raw_lines, &errs);
        //         return;
        //     }
        // };

        //println!("\n Removing leading labels.");
        // for tk_ln in &self.tokenized_lines {
        //     let line_offset = tk_ln.rel_addr;

        //     let line = match tk_ln.tokens.strip_prefix(&[Token::Label(format!(""))]) {
        //         Some(without_label) => without_label,
        //         None => &tk_ln.tokens,
        //     };
        //     //println!("{line_offset} \t {line:?}");
        //     match self.parse_single_instr(line.to_vec(), line_offset) {
        //         None => {}
        //         Some(word) => {
        //             instructions.push(word);
        //         }
        //     }
        // }

        // let filtered_lines = (&self.tokenized_lines)
        //     .into_iter()
        //     .map(|(line, line_offset)| {line.into_iter().map())});
        let vm = &mut self.vm;
        self.orig = img.origin;
        vm.set_program_origin(self.orig);

        trap_instructions.map(|x| {
            for trap in x {
                vm.write_memory(trap.trap_vector, trap.origin);
                println!(
                    "Trap vector: 0x{:x}, value: 0x:{:x} ",
                    trap.trap_vector, trap.origin
                );
                for (addr, val) in trap.memory_writes {
                    vm.write_memory(addr, val);
                }
                vm.load_binary_into_memory(trap.instructions, trap.origin);
            }
        });

        //vm.load_binary_into_memory(instructions, self.orig);
        for w in &img.instructions {
            vm.write_memory(w.rel_addr + img.origin, w.value);
            println!(
                "{}",
                InstructionSet::dissasemble_memory(
                    w.value,
                    Some(w.rel_addr + img.origin),
                    None,
                    None
                )
            );
        }

        for w in &img.data {
            //println!("Writing DATA: 0x{:x} <= {:x}", w.rel_addr + img.origin, w.value);
            vm.write_memory(w.rel_addr + img.origin, w.value);
            println!(
                "{}",
                InstructionSet::dissasemble_memory(
                    w.value,
                    Some(w.rel_addr + img.origin),
                    None,
                    None
                )
            );
        }

        vm.run_io_thread();
        thread::sleep(time::Duration::from_millis(50));
        loop {
            vm.fetch();
            vm.decode();
            vm.execute(Some(&img.symbol_table));

            if !vm.run {
                thread::sleep(time::Duration::from_millis(10));
                // print!("Ending VM instance...");
                // print!("Done.\n");

                break;
            }
            //Term::stdout().read_char();
        }
    }

    pub fn parse_instructions(&mut self) -> Result<Vec<u16>, Vec<AsmblrErr>> {
        let mut instructions: Vec<u16> = Vec::new();
        let mut errors = Vec::new();

        //println!("\n Removing leading labels.");
        for tk_ln in &self.tokenized_lines {
            let line = match tk_ln.tokens.strip_prefix(&[Token::Label(format!(""))]) {
                Some(without_label) => without_label,
                None => &tk_ln.tokens,
            };
            //println!("{line_offset} \t {line:?}");
            match self.parse_single_instr(line.to_vec(), tk_ln.rel_addr) {
                Ok(instr) => match instr {
                    None => {}
                    Some(word) => {
                        instructions.push(word);
                    }
                },
                Err(msg) => errors.push(AsmblrErr::new(tk_ln.src_ln_number, msg)),
            }
        }

        if !errors.is_empty() {
            //println!("{errors:?}");
            return Err(errors);
        }

        // let filtered_lines = (&self.tokenized_lines)
        //     .into_iter()
        //     .map(|(line, line_offset)| {line.into_iter().map())});
        Ok(instructions)
    }

    fn parse_single_instr(&self, tokens: Vec<Token>, rel_addr: u16) -> Result<Option<u16>, String> {
        let target_instruction: &InstrDef; // = &InstrDef::new(OP::RES, 0, vec![]);

        if tokens.starts_with(&[Token::Instruction(format!(""))]) {
            //Find which instruction it is;
            match tokens.first().unwrap() {
                Token::Instruction(instr) => {
                    //println!("{:03} Searching for instruction '{instr}'.", line_offset);
                    target_instruction = &self.instruction_set[&instr.to_ascii_uppercase()];
                    //.expect("Undefined instruction {instr}");
                }
                // /Token::Directive(dir) => {
                //     println!("Ignoring directive .{dir} (line {line_offset})");
                // },
                _ => panic!("Expected instr, fatal error."),
            }
        } else {
            //println!("Ignoring line {line:?}");
            return Ok(None);
        }

        //Get line's paramaters
        let mut args: Vec<Token> = vec![];
        let mut arg_index = 0;

        //let mut previous: Vec<Token> = Vec::new();

        for i in 0..tokens.len() {
            if i == 0 {
                //Ignore instruction
                continue;
            }

            // println!("Token: '{:?}', i = {i}, arg_index = {arg_index}", line[i]);

            if arg_index == target_instruction.params.len() {
                return Err(format!(
                    "Expected {} operands, found {}. \t[{:?}]",
                    target_instruction.params.len(),
                    arg_index,
                    tokens
                ));
            }

            if (i % 2) == 0 && !tokens[i].is(&Token::Comma) {
                return Err(format!("Expecting comma between params. {:?}", tokens));
            }

            if tokens[i].is(&Token::Comma) {
                continue;
            }

            if i % 2 != 0 && !tokens[i].is_valid_arg(&target_instruction.params[arg_index]) {
                return Err(format!(
                    "Expected {:?} for instruction '{}', found token '{:?}'",
                    target_instruction.params[arg_index], target_instruction.opcode, tokens[i]
                ));
            }

            //previous.push(line[i].clone());

            args.push(tokens[i].clone());
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
            return Err(format!(
                "Expected {} arguments, but found {}.",
                target_instruction.params.len(),
                args.len()
            ));
        }
        for k in 0..target_instruction.params.len() {
            match target_instruction.params[k] {
                Param::Register(pos) => match args[k] {
                    Token::Register(r) => word += r << pos,
                    _ => return Err(format!("Expected register as argument")),
                },

                Param::RegisterORImm5 => {
                    match &args[k] {
                        Token::DecimalLiteral(val) | Token::HexLiteral(val) => {
                            let mut num: u16 = val.value;
                            // println!("|Imm5|: {num:016b} ({num}))");
                            if num > 2u16.pow(4) {
                                return Err(format!("Invalid imm5, MAX +/-15"));
                            }

                            if matches!(val.sign, Sign::MINUS) {
                                //println!("Negative imm5. {num}");
                                num = truncate_to_bit(invert_sign(num), 5) /*+ flag_set_mask(5)*/;
                            }
                            word += num;
                            word = set_flag_true(word, 5);
                            //word += flag_set_mask(5);
                        }
                        Token::Register(reg) => {
                            word += reg;
                        }
                        _ => return Err(format!("Expected register or Imm5 (number)")),
                    }
                }

                Param::Bits(bits) => {
                    //match &args[k] {
                    if let Token::DecimalLiteral(val) | Token::HexLiteral(val) = &args[k] {
                        let mut num: u16 = val.value;
                        if matches!(val.sign, Sign::MINUS) {
                            num = truncate_to_bit(invert_sign(num), bits);
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
                                if sym.name.to_ascii_uppercase() == *lbl.to_ascii_uppercase() {
                                    symbol_value = Some(sym.rel_addr);
                                }
                            }

                            if symbol_value == None {
                                return Err(format!("Undefined label '{lbl}'"));
                            }
                            let symbol_value = symbol_value.unwrap();

                            //PC-Offset-9
                            // println!(
                            //     "Label: {symbol_value}, PC: {rel_addr}. L-PC = {}",
                            //     as_negative_i16(add_2s_complement(
                            //         symbol_value,
                            //         invert_sign(rel_addr)
                            //     ))
                            // );

                            let pc_offset_9 = truncate_to_n_bit(add_2s_complement(
                                    symbol_value,
                                    invert_sign(rel_addr+1),),10) /*<< 7
                                    >> 7*/
                            ;
                            //WARNING

                            // if is_negative(pc_offset_9){
                            //     pc_offset_9 = (set_flag_true(pc_offset_9, 8) << 7 ) >> 7;
                            // }

                            // println!(
                            //     "PC-offset 9 = {}",
                            //     as_negative_i16(pc_offset_9)
                            // );
                            word += pc_offset_9;
                        }

                        _ => return Err(format!("Expected label as argument")),
                    }
                }
                
            }
        }

        println!("Instr: {word:016b}");
        Ok(Some(word))
    }
}

pub type SymbolTable = Vec<Symbol>;

pub fn is_instruction(s: &str) -> bool {
    vec![
        "AND", "ADD", "NOT", "BR", "BRZ", "BRP", "BRN", "BRNZ", "BRNZP", "BRNP", "BRZP", "LD",
        "LDI", "LDR", "ST", "STR", "STI", "TRAP", "JMP", "RET", "JSR", "JSSR", "LEA", "HALT", "IN","OUT", "PUTS",
    ]
    .contains(&s.to_ascii_uppercase().as_str())
}

use std::collections::HashMap;

#[derive(Debug)]
pub struct InstrDef {
    opcode: u16,
    flags_word: u16,
    not_flags_word: u16, //bits that must be zero
    params: Vec<Param>,
}

impl InstrDef {
    fn new(opcode: virtual_machine::OP, flags_word: u16, not_flags_word:u16, params: Vec<Param>) -> Self {
        InstrDef {
            opcode: (opcode as u16) << 12,
            flags_word,
            not_flags_word,
            params,
        }
    }
}

pub struct InstructionSet {
    // lines: Vec<String>,
    // symbolTable: SymbolTable,
    // instruction_set: HashMap<String, InstrDef>,
}
use virtual_machine::OP;

impl InstructionSet {
    fn define_instruction_set() -> HashMap<String, InstrDef> {
        let mut instr_set = HashMap::new();
        instr_set.insert(
            String::from("ADD"),
            InstrDef::new(
                OP::ADD,
                0,
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
                0,
                vec![
                    Param::Register(9),
                    Param::Register(6),
                    Param::RegisterORImm5,
                ],
            ),
        );

        let n = flag_set_mask(11);
        let z = flag_set_mask(10);
        let p = flag_set_mask(9);

        instr_set.insert(
            String::from("BRN"),
            InstrDef::new(OP::BR, flag_set_mask(11),0, vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRZ"),
            InstrDef::new(OP::BR, flag_set_mask(10),0, vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRP"),
            InstrDef::new(OP::BR, flag_set_mask(9), 0,vec![Param::Label]),
        );

        instr_set.insert(
            String::from("BRNZ"),
            InstrDef::new(
                OP::BR,
                flag_set_mask(11) + flag_set_mask(10), 
                0,
                vec![Param::Label],
            ),
        );

        instr_set.insert(
            String::from("BRNP"),
            InstrDef::new(
                OP::BR,
                flag_set_mask(11) + flag_set_mask(9),
                0,
                vec![Param::Label],
            ),
        );

        instr_set.insert(
            String::from("BRZP"),
            InstrDef::new(
                OP::BR,
                flag_set_mask(10) + flag_set_mask(9),
                0,
                vec![Param::Label],
            ),
        );

        instr_set.insert(
            String::from("BRNZP"),
            InstrDef::new(
                OP::BR,
                flag_set_mask(10) + flag_set_mask(9) + flag_set_mask(11),
                0,
                vec![Param::Label],
            ),
        );

        instr_set.insert(
            String::from("BR"),
            InstrDef::new(OP::BR, 0, n+z+p, vec![Param::Label]),
        );

        //let nzp = flag_set_mask(10) + flag_set_mask(9) + flag_set_mask(11);
        //Define RET before JMP so dissasem .find defaults to RET when JMP R7 is encountered
        instr_set.insert(
            String::from("RET"),
            InstrDef::new(OP::JMP /*OP::BR */, 0b0000_000_111_000000, 0, vec![]),
        );

        instr_set.insert(
            String::from("JMP"),
            InstrDef::new(OP::JMP, 0, 0b0000_000_111_000000,vec![Param::Register(6)]),
        );

        instr_set.insert(
            String::from("JSR"),
            InstrDef::new(OP::JSR, flag_set_mask(11), 0, vec![Param::Label]),
        );

        instr_set.insert(
            String::from("JSSR"),
            InstrDef::new(OP::JSR, 0, flag_set_mask(11),vec![Param::Register(6)]),
        );

        instr_set.insert(
            String::from("LD"),
            InstrDef::new(OP::LD, 0, 0, vec![Param::Register(9), Param::Label]),
        );

        instr_set.insert(
            String::from("LDI"),
            InstrDef::new(OP::LDI, 0, 0, vec![Param::Register(9), Param::Label]),
        );

        instr_set.insert(
            String::from("LDR"),
            InstrDef::new(
                OP::LDR,
                0,
                0,
                vec![Param::Register(9), Param::Register(6), Param::Bits(6)],
            ),
        );

        instr_set.insert(
            String::from("LEA"),
            InstrDef::new(OP::LEA, 0, 0, vec![Param::Register(9), Param::Label]),
        );

        instr_set.insert(
            String::from("NOT"),
            InstrDef::new(
                OP::NOT,
                0b11_1111,
                0,
                vec![Param::Register(9), Param::Register(6)],
            ),
        );

        instr_set.insert(
            String::from("RET"),
            InstrDef::new(OP::JMP, 0b111 << 6 /*set register to R7 */, 0, vec![]),
        );
        instr_set.insert(String::from("RTI"), InstrDef::new(OP::RTI, 0, 0, vec![]));

        instr_set.insert(
            String::from("ST"),
            InstrDef::new(OP::ST, 0, 0, vec![Param::Register(9), Param::Label]),
        );
        instr_set.insert(
            String::from("STI"),
            InstrDef::new(OP::STI, 0, 0, vec![Param::Register(9), Param::Label]),
        );
        instr_set.insert(
            String::from("STR"),
            InstrDef::new(
                OP::STR,
                0,
                0,
                vec![Param::Register(9), Param::Register(6), Param::Bits(6)],
            ),
        );

        
        instr_set.insert(String::from("IN"), InstrDef::new(OP::TRAP, 0x23, 0, vec![]));
        instr_set.insert(String::from("OUT"), InstrDef::new(OP::TRAP, 0x21, 0, vec![]));
        instr_set.insert(String::from("PUTS"), InstrDef::new(OP::TRAP, 0x22, 0, vec![])); 
        instr_set.insert(String::from("HALT"), InstrDef::new(OP::RES, 0, 0, vec![]));
        instr_set.insert(
            String::from("TRAP"),
            InstrDef::new(OP::TRAP, 0, 0, vec![Param::Bits(8)]),
        );
        instr_set
    }

    pub fn dissasemble_memory(
        mem: u16,
        incremented_program_counter: Option<u16>,
        symbol_table: Option<&SymbolTable>,
        instr_set: Option<HashMap<String, InstrDef>>,
    ) -> String {
        let instr_set = match instr_set {
            None => Self::define_instruction_set(),
            Some(set) => set,
        };

        let mut disassem_text = String::new();

        //(1) Resolve opcode
        let opcode = instructions::get_opcode_16bit(mem);
        let possible_instructions: Vec<(&String, &InstrDef)> = instr_set
            .iter()
            .filter(|(_, definition)| (**definition).opcode == opcode)
            .collect();
        //println!("Possible instrs: {:#?}", possible_instructions);
        //(2) resolve variant
        let (variant_name, variant_definition) =
            match possible_instructions.iter().find(|(_, definition)| {
                (mem & definition.flags_word) == definition.flags_word 
                && mem & definition.not_flags_word == 0
                    /*&& !(definition.flags_word == 0 && ((mem & 0b0000_100_0000_00000) == 0))*/
            }) {
                None => {
                    println!("[DISASSEM]\tUnable to resolve instruction variant.");
                    return disassem_text;
                }
                Some(definition) => definition,
            };

        disassem_text += &format!("{variant_name}\t");
        //(3) Parse out arguments according to definition
        for (param_index, param) in variant_definition.params.iter().enumerate() {
            use Param::*;
            match &param {
                &Bits(n) => {
                    let number = as_negative_i32(instructions::get_sign_ext_value(mem, *n));
                    disassem_text += &format!("0x{number:x}");
                }
                &Register(starting_at) => {
                    let reg = instructions::get_register_at(mem, (*starting_at, starting_at + 2));
                    disassem_text += &format!("R{reg}");
                }
                &Label => {
                    //Pc-offset_9
                    let pc_offset_9 = instructions::get_sign_ext_value(mem, 9);
                    //disassem_text += &format!("0x{pc_offset_9:x}");

                    if incremented_program_counter.is_some() && symbol_table.is_some() {
                        let instr_addr = incremented_program_counter.unwrap();
                        let symbol_table = symbol_table.unwrap();
                        match symbol_table.iter().find(|symbol| {
                            symbol.abs_addr == add_2s_complement(instr_addr, pc_offset_9)
                        }) {
                            Some(symbol) => disassem_text += &format!("[{}]", symbol.name),
                            None => {
                                disassem_text += &format!(
                                    "[0x{:04x}]",
                                    add_2s_complement(instr_addr, pc_offset_9)
                                )
                            } /*&format!("[0x{pc_offset_9:x}]", )*/,
                        }
                    } else {
                        let pc_offset_9 = as_negative_i32(pc_offset_9);
                        disassem_text += &format!("0x{pc_offset_9:x}");
                    }
                }
                //Bit 5 tell difference
                &RegisterORImm5 => {
                    if flag_is_set(mem, 5) {
                        //imm 5
                        let number = as_negative_i32(instructions::get_sign_ext_value(mem, 5));
                        disassem_text += &format!("#{number}");
                    } else {
                        let reg = instructions::get_register_at(mem, (0, 2));
                        disassem_text += &format!("R{reg}");
                    }
                } // &Imm5 => {
                  //     let number = as_negative_i32(instructions::get_sign_ext_value(mem, 5));
                  //     disassem_text += &format!("0x{number:x}");
                  // }
            }
            if param_index + 1 < variant_definition.params.len() {
                disassem_text += ", ";
            }
        }

        disassem_text
    }
}

#[derive(Debug)]
pub enum Param {
    Bits(u16),
    Register(u16), /*Lower bit [val -> val+2] */
    Label,
    //Label6bit,
    RegisterORImm5,
    //Imm5,
}
