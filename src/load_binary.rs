use crate::binary_utils;
use crate::assembler::*;
use crate::binary_utils::as_negative_i32;
use assemble::*;
use tokenizer::Token;
use crate::error;
use std::fs::File;
use std::io::Read;
use std::io::Write;

pub enum Endian{
    Big,
    Little,
}

pub fn read_binary_from_file(path: &str, endian: Endian) -> Result<Vec<u16>, error::FileLoadError> {
    let file_open_result = File::open(path);

    let mut file = match file_open_result {
        Ok(f) => f,
        Err(e) => {
            dbg!(e);
            return Err(error::FileLoadError::FsOpenFailed);
        }
    };

    let mut contents: Vec<u8> = Vec::new();
    let file_read_result = file.read_to_end(&mut contents);

    match file_read_result {
        Ok(_) => {}
        Err(_) => {
            return Err(error::FileLoadError::FsReadFailed);
        }
    };

    if contents.len() % 2 != 0 || contents.is_empty() {
        //Valid LC-3 binary should always contain an even number of bytes, since each instruction is 16-bits (=2 bytes)
        return Err(error::FileLoadError::InvalidBinary);
    }

    let mut merged_contents: Vec<u16> = Vec::new();

    let mut first_halves = Vec::new();
    let mut second_halves = Vec::new();

    for i in 0..contents.len() {
        if i % 2 == 0 {
            first_halves.push(contents[i]);
        } else {
            second_halves.push(contents[i]);
        }
    }

    for (f, s) in first_halves.iter().zip(second_halves.iter()) {
        //println!("")
        match endian{
            Endian::Little => merged_contents.push(binary_utils::merge_bytes(*s, *f)),
            Endian::Big => merged_contents.push(binary_utils::merge_bytes(*s, *f)),
        }
    }

    Ok(merged_contents)
}

// pub fn binary_to_img(bin: Vec<u16>) -> assemble::ExecutableImage{
//     assemble::ExecutableImage{
//         origin: bin[0],
//         name: String::new(),
//         instructions:
//     }
// }

pub fn write_binary_to_file(path: &str, img: &assemble::ExecutableImage) -> Result<usize, error::FileLoadError>{
    let file_open_result = File::create(path);

    let mut file = match file_open_result {
        Ok(f) => f,
        Err(e) => {
            dbg!(e);
            return Err(error::FileLoadError::FsOpenFailed);
        }
    };

    let mut contents: Vec<u16> = Vec::new();
    contents.push(img.origin);
    contents = [contents, img.instructions.iter().map(|write| {
        write.value
    }).collect()].concat();
    contents = [contents, img.data.iter().map(|write| {
        write.value
    }).collect()].concat();
    
    let contents: Vec<[u8; 2]> = contents.iter().map(|word| {word.to_le_bytes()}).collect();
    let mut size = 0;
    for c in &contents{
        
        size += file.write(&[c[1]]).expect("Error writing bytes to file");
        size += file.write(&[c[0]]).expect("Error writing bytes to file");
       
    }
    file.write(&[0u8,0u8]).expect("Error writing bytes to file");
    Ok(size)
}

pub fn write_symbols_to_file(path: &str, img: &assemble::ExecutableImage) -> Result<usize, error::FileLoadError>{
    let file_open_result = File::create(path);

    let mut file = match file_open_result {
        Ok(f) => f,
        Err(e) => {
            dbg!(e);
            return Err(error::FileLoadError::FsOpenFailed);
        }
    };
    let mut symbol_count = 0;
    //file.write(format!(";{:20}\t{:4}\t\t{}\n", "Name","Addr","Status").as_bytes());
    for symbol in &img.symbol_table{
        file.write(format!("{:20}\t#{:04}\tx{:04x}\t\t{}\n", symbol.name, as_negative_i32(symbol.rel_addr), symbol.abs_addr, symbol.status.clone() as u16).as_bytes()).expect("Failure writing symbol table to file.");
        symbol_count += 1;
    }
    Ok(symbol_count)
}

pub fn read_symbols_from_file(path: &str) -> Result<Vec<assemble::Symbol>, error::FileLoadError> {
    let file_open_result = File::open(path);
    let mut symbols = Vec::new();

    let mut file = match file_open_result {
        Ok(f) => f,
        Err(e) => {
            dbg!(e);
            return Err(error::FileLoadError::FsOpenFailed);
        }
    };

    let mut contents = String::new();
    let file_read_result = file.read_to_string(&mut contents);

    match file_read_result {
        Ok(_) => {}
        Err(_) => {
            return Err(error::FileLoadError::FsReadFailed);
        }
    };

    let contents = contents.lines();
    for line in contents.into_iter(){
        println!("{line}");
        let mut tokens = Token::tokenize_str(line);
        //println!("{tokens:?}");
        if tokens.len() != 4{
            return Err(error::FileLoadError::InvalidSymbols);
        }

        let mut stream = tokens.iter_mut();
        let name = match stream.next() {
            None => return Err(error::FileLoadError::InvalidSymbols),
            Some(token) => match token{
                Token::Label(label) => label,
                _ => {
                    println!("Expected string");
                    return Err(error::FileLoadError::InvalidSymbols);
                }
            },
        }.to_string();

        let rel_addr = match stream.next() {
            None => return Err(error::FileLoadError::InvalidSymbols),
            Some(token) => match token{
                Token::DecimalLiteral(addr) => addr.value,
                _ => return Err(error::FileLoadError::InvalidSymbols)
            },
        };

        let abs_addr = match stream.next() {
            None => return Err(error::FileLoadError::InvalidSymbols),
            Some(token) => match token{
                Token::HexLiteral(addr) => addr.value,
                _ => return Err(error::FileLoadError::InvalidSymbols)
            },
        };
        let status = match stream.next() {
            None => return Err(error::FileLoadError::InvalidSymbols),
            Some(token) => match token{
                Token::DecimalLiteral(number) => {
                    match number.value{
                        0 => SymbolStatus::Private,
                        1 => SymbolStatus::Export,
                        2 => SymbolStatus::Import,
                        _ => return Err(error::FileLoadError::InvalidSymbols),
                    }
                } 
                _ => return Err(error::FileLoadError::InvalidSymbols)
            },
        };
        symbols.push(Symbol { name, rel_addr: rel_addr, abs_addr, src_ln_number: 0, size_in_words: 0, status })
        
    }
   

    Ok(symbols)
}