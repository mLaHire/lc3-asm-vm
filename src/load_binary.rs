use crate::binary_utils;
use crate::assembler::*;
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
    Ok(size)
}

// pub fn write_symbols_to_file(path: &str, img: &assemble::ExecutableImage) -> Result<usize, error::FileLoadError>{
//     let file_open_result = File::create(path);

//     let mut file = match file_open_result {
//         Ok(f) => f,
//         Err(e) => {
//             dbg!(e);
//             return Err(error::FileLoadError::FsOpenFailed);
//         }
//     };

//     let mut contents: Vec<u16> = Vec::new();
//     contents.push(img.origin);
//     contents = [contents, img.instructions.iter().map(|write| {
//         write.value
//     }).collect()].concat();
//     contents = [contents, img.data.iter().map(|write| {
//         write.value
//     }).collect()].concat();
    
//     let contents: Vec<[u8; 2]> = contents.iter().map(|word| {word.to_le_bytes()}).collect();
//     let mut size = 0;
//     for c in &contents{
        
//         size += file.write(&[c[1]]).expect("Error writing bytes to file");
//         size += file.write(&[c[0]]).expect("Error writing bytes to file");
       
//     }
//     Ok(size)
// }