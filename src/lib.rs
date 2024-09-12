pub mod assembler;
pub mod binary_utils;
pub mod error;
pub mod load_binary;
pub mod virtual_machine;
use std::process::Output;

use assemble::*;
use assembler::*;

pub fn parse_arguments(args: Vec<String>) {
    if args.len() < 3 {
        panic!("Too few arguments");
    }

    //ignore args[0]

    match args[1].to_ascii_lowercase().as_str() {
        "assemble" => {
            let src_files = vec![args[2].clone()];

            let src_file0 = src_files[0].clone(); //Guaranteed to exist

            if args.len() > 3 {
                for arg in 3..args.len() {
                    //process flags
                }
            }
            let mut output_file0 = match src_files[0].strip_suffix(".asm") {
                None => src_files[0].clone(),
                Some(stripped) => stripped.to_string(),
            };

            output_file0.push_str(".obj");

            assemble(src_file0, output_file0, String::new());
        }
        "link" => {}
        "load" => {}
        "help" => {}
        _ => panic!("Invalid argument."),
    }
}

pub fn assemble(src_file: String, output_file: String, asm_flags: String) {
    let mut asm = Assembler::new(&src_file);
    asm.load();
    //let result = asm.assemble();

    let img = match asm.assemble() {
        Ok(img) => img,
        Err(errors) => {
            error::AsmblrErr::display(&src_file, &asm.raw_lines, &errors);
            eprintln!("\n[ASM]\tAssembly failed, {} error(s).", errors.len());
            return;
        }
    };

    let _ = match load_binary::write_binary_to_file(
        &format!("{}", output_file),
        &img,
    ) {
        Ok(size) => println!("[OK]\tWrote {size} bytes to {}", output_file),
        Err(e) => panic!("[FAIL]\t{:?}", e),
    };
}

pub fn link_load_and_execute(src_file: String, link_files: Vec<String>, vm_flags: String) {
    // Set up VM context

    //Load trap files according to config

    //Modify executable
}
