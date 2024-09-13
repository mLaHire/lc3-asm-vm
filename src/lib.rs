pub mod assembler;
pub mod binary_utils;
pub mod error;
pub mod file_io;
pub mod virtual_machine;
pub mod cli;
use std::{process::Output, thread, time};

use assemble::*;
use assembler::*;
use binary_utils::*;
use file_io::*;
use virtual_machine::*;
use error::CliError;



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

pub fn parse_arguments(args: Vec<String>) -> Result<(), CliError> {
    if args.len() < 3 {
        return Err(CliError::new("Expected at least 3 arguments"))
    }

    //ignore args[0]

    match args[1].to_ascii_lowercase().as_str() {
        "asm" => {
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
                                return Err(CliError::new("Expected files to link after '--link'"))
                            }
                            for k in arg_no+1..args.len() {
                                external_files.push(&args[k]);
                            }
                            break;
                        }
                        _ => return Err(CliError::new("Expected flags or output file name after 'asm'")),
                    }
                }
            }
            let mut output_file0 = match src_files[0].strip_suffix(".asm") {
                None => src_files[0].clone(),
                Some(stripped) => stripped.to_string(),
            };

            output_file0.push_str(".obj");

            cli_assemble(src_file0, output_file0, external_files, flags);
        }
        "link" => {}
        "load" => {
            let mut src_files: Vec<&String> = args
                .iter()
                .enumerate()
                .filter(|(i, val)| *i > 1)
                .map(|(i, val)| val)
                .collect();

            if src_files.len() == 0{ 
                return Err(CliError::new(&format!("Expected files to load after 'load'")));
            }

            let link_files = src_files.split_off(1);

            cli_link_load_and_execute(src_files[0], link_files, None);
        }
        "help" => {}
        _ => return Err(CliError::new(&format!("Invalid argument '{}'", args[1]))),
    }
    Ok(())
}

pub fn cli_assemble(
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

    let _ = match file_io::write_binary_to_file(&format!("{}", output_file), &img) {
        Ok(size) => println!("[OK]\tWrote {size} bytes to {}", output_file),
        Err(e) => panic!("[FAIL]\t{:?}", e),
    };

    if flags.output_symbol_file {
        let _ = match file_io::write_symbols_to_file(&format!("{output_file}.sym"), &img) {
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

pub fn cli_link_load_and_execute(src_file: &str, link_files: Vec<&String>, vm_flags: Option<bool>) {
    // Set up VM context
    let verbose_log = false;
    

    //Load trap files according to config
    let putc_x21 = TrapInstruction::new(TRAP_DIR_PATH, "putc", 0x21);
    let puts_x22 = TrapInstruction::new(TRAP_DIR_PATH, "puts", 0x22);
    let getc_x23 = TrapInstruction::new(TRAP_DIR_PATH, "getc", 0x23);
    let halt_x25 = TrapInstruction::new(TRAP_DIR_PATH, "halt", 0x25);

    let mut trap_instructions = vec![putc_x21, puts_x22, getc_x23, halt_x25];

    //load files
    let src_img = match read_exectuable_img_from_file(&src_file, Endian::Little){
        Err(e) => {
            eprintln!("Error loading executable image '{}', {e:?}", src_file);
            return;
        }
        Ok(img) => img,
    };

    let mut executable_images = vec![src_img];

    for link_file in link_files{
        let img = match read_exectuable_img_from_file(&link_file, Endian::Little){
            Err(e) => {
                eprintln!("Error linking executable image '{}', {e:?}", src_file);
                return;
            }
            Ok(img) => img,
        }; 
        executable_images.push(ExecutableImageIn{
            data: img.data,
            origin: img.origin,
        });
    }

    if  let Some(((min1, max1), (min2, max2))) = ExecutableImageIn::images_overlap(&executable_images){
        eprintln!("Error linking executable images, images have overlapping memory locations:\t [0x{:04}, 0x{:04}] and [0x{:04}, 0x{:04}]", min1, max1, min2, max2);
        return;
    }

    let mut ctx = VirtualMachine::new();
    ctx.set_program_origin(executable_images[0].origin);
    
    for trap in trap_instructions {
        ctx.write_memory(trap.trap_vector, trap.origin);
        // println!(
        //     "Trap vector: 0x{:x}, value: 0x:{:x} ",
        //     trap.trap_vector, trap.origin
        // );
        for (addr, val) in trap.memory_writes {
            ctx.write_memory(addr, val);
        }
        ctx.load_binary_into_memory(trap.instructions, trap.origin);
    }

    for img in executable_images{
        for (rel_addr, word) in img.data.iter().enumerate(){
            ctx.write_memory(img.origin+rel_addr as u16 , *word);
            println!("0x{:04x} <- {:016b}", img.origin+rel_addr as u16, word);
        }
    }

    ctx.run_io_thread();
    thread::sleep(time::Duration::from_millis(50));
    loop {
        ctx.fetch();
        ctx.decode();
        ctx.execute(None); //IF IMPORT SYMBOL TABLE

        if !flag_is_set(ctx.read_memory(ctx.mcr_address), 15) {
            thread::sleep(time::Duration::from_millis(10));
            print!("Ending ctx instance... ");
            ctx.write_memory(ctx.kbsr_address, set_flag_true(0, 14));
            ctx.write_memory(ctx.dsr_address, set_flag_true(0, 14));
            print!("Press any key to exit...");
            while flag_is_set(ctx.read_memory(ctx.kbsr_address), 14)
                || flag_is_set(ctx.read_memory(ctx.dsr_address), 14)
            {
                //wait for Input server to terminate.
                
                thread::sleep(time::Duration::from_millis(10));
            }

            print!("Done.\n");

            break;
        }
        //Term::stdout().read_char();
    }
}
