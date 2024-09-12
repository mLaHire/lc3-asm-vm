pub mod assembler;
pub mod binary_utils;
pub mod error;
pub mod load_binary;
pub mod virtual_machine;
use std::process::Output;

use assemble::*;
use assembler::*;

pub struct AssemblerFlags {
    pub case_insensitive_labels: bool,
    pub output_symbol_file: bool,
    pub verbose_log: bool,
}

impl AssemblerFlags {
    pub fn new() -> Self {
        AssemblerFlags {
            case_insensitive_labels: true,
            output_symbol_file: true,
            verbose_log: false,
        }
    }

    pub fn case_sensitive_labels(&mut self) -> &mut Self {
        self.case_insensitive_labels = false;
        self
    }

    pub fn set_symbol_file(&mut self, flag: bool) -> &mut Self {
        self.output_symbol_file = flag;
        self
    }

    pub fn set_verbose_log(&mut self, flag: bool) -> &mut Self {
        self.verbose_log = flag;
        self
    }
}

pub fn parse_arguments(args: Vec<String>) {
    if args.len() < 3 {
        panic!("Too few arguments");
    }

    //ignore args[0]

    match args[1].to_ascii_lowercase().as_str() {
        "assemble" => {
            let src_files = vec![args[2].clone()];
            let src_file0 = src_files[0].clone(); //Guaranteed to exist
            let mut flags = AssemblerFlags::new();

            let mut external_files: Vec<&str> = vec![];

            if args.len() > 3 {
                for mut arg_no in 3..args.len() {
                    match args[arg_no].as_str() {
                        "--case-sensitive" => {
                            flags.case_sensitive_labels();
                        }
                        "--no-sym-file" => {
                            flags.set_symbol_file(false);
                        }

                        "--verbose-log" => {
                            flags.set_verbose_log(true);
                        }

                        "--link" => {
                            if arg_no+1 == args.len() {
                                panic!("Expected files to link.")
                            }
                            for k in arg_no+1..args.len() {
                                external_files.push(&args[k]);
                            }
                            break;
                            arg_no = args.len();
                        }
                        _ => panic!("Error: Expected flags or output file name. "),
                    }
                }
            }
            let mut output_file0 = match src_files[0].strip_suffix(".asm") {
                None => src_files[0].clone(),
                Some(stripped) => stripped.to_string(),
            };

            output_file0.push_str(".obj");

            assemble(src_file0, output_file0, external_files, flags);
        }
        "link" => {}
        "load" => {
            let mut src_files: Vec<&String> = args
                .iter()
                .enumerate()
                .filter(|(i, val)| *i > 1)
                .map(|(i, val)| val)
                .collect();

            
        }
        "help" => {}
        _ => panic!("Error: Invalid argument."),
    }
}

pub fn assemble(
    src_file: String,
    output_file: String,
    external_files: Vec<&str>,
    flags: AssemblerFlags,
) {
    let mut asm = Assembler::new(&src_file);
    asm.ignore_case_for_labels(flags.case_insensitive_labels);
    asm.verbose_log = flags.verbose_log;
    asm.load();

    //let result = asm.assemble();

    let img = match asm.assemble(external_files) {
        Ok(img) => img,
        Err(errors) => {
            error::AsmblrErr::display(&src_file, &asm.raw_lines, &errors);
            eprintln!("\n[ASM]\tAssembly failed, {} error(s).", errors.len());
            return;
        }
    };

    let _ = match load_binary::write_binary_to_file(&format!("{}", output_file), &img) {
        Ok(size) => println!("[OK]\tWrote {size} bytes to {}", output_file),
        Err(e) => panic!("[FAIL]\t{:?}", e),
    };

    if flags.output_symbol_file {
        let _ = match load_binary::write_symbols_to_file(&format!("{output_file}.sym"), &img) {
            Ok(_) => {
                println!(
                    "[OK]\tWrote {} symbols to {output_file}.sym:",
                    img.symbol_table.len()
                );
                println!(
                    "{:>3}\tprivate symbol(s).",
                    img.symbol_table
                        .iter()
                        .filter(|s| matches!(s.status, SymbolStatus::Private))
                        .count()
                );
                println!(
                    "{:>3}\texported symbol(s).",
                    img.symbol_table
                        .iter()
                        .filter(|s| matches!(s.status, SymbolStatus::Export))
                        .count()
                );
                println!(
                    "{:>3}\timported symbol(s).",
                    img.symbol_table
                        .iter()
                        .filter(|s| matches!(s.status, SymbolStatus::Import))
                        .count()
                );
            }
            Err(e) => panic!("[FAIL]\t{:?}", e),
        };
    }
}

pub fn link_load_and_execute(src_file: String, link_files: Vec<String>, vm_flags: String) {
    // Set up VM context

    //Load trap files according to config

    //Modify executable
}
