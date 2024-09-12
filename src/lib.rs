pub mod assembler;
pub mod binary_utils;
pub mod error;
pub mod load_binary;
pub mod virtual_machine;
use assembler::*;

pub fn parse_arguments(args: Vec<String>) {
    if args.len() < 3{ 
        panic!("Too few arguments");
    }

    match args[0].to_ascii_lowercase().as_str(){
        "assemble" => {},
        "link" => {},
        "load" => {},
        "help" => {},
        _ => panic!("Invalid argument.")
    }
}

pub fn assemble(src_file: String, output_file: Option<String>, asm_flags: String){

}

pub fn link_load_and_execute(src_file: String, link_files: Vec<String>, vm_flags: String){
    // Set up VM context 

    //Load trap files according to config 

    //Modify executable 
}