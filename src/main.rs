use crate::assembler::*;
use assemble::*;
use console::Term;
use lc3_asm_vm::parse_arguments;
use std::path::Path;
use std::time::Instant;
pub mod assembler;
pub mod binary_utils;
pub mod error;
pub mod file_io;
pub mod virtual_machine;

//
//
//
//fn compile(source_file: &Path, external_symbol_files: Vec<&Path>) {}

enum CliRequestType {
    Assemble,
    LinkLoad,
    Flag(String),
}

struct CliRequest {}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    //print!("\nArguments: {args:?}\n");
    if args.len() > 1 && args[1] == "--old"{
        main_obsolete();
        return;
    }
    match parse_arguments(args){
        Ok(_) => {},
        Err(cli_error) => cli_error.dipsplay(),
    }
}

fn main_obsolete() {
   // let args: Vec<String> = std::env::args().collect();
    //print!("Arguments: {args:?}");
    //return;

    let putc_x21 = TrapInstruction::new("putc", 0x21);
    let puts_x22 = TrapInstruction::new("puts", 0x22);
    let getc_x23 = TrapInstruction::new("getc", 0x23);
    let halt_x25 = TrapInstruction::new("halt", 0x25);
    let term = Term::stdout();

    print!("Enter local file path: ./src/asm_files/");
    let buffer = match term.read_line() {
        Ok(p) => p,
        Err(e) => panic!("{e}"),
    }
    .trim()
    .to_owned();

    let mut path = String::from("./src/asm_files/");
    path.push_str(&buffer);
    print!("Debug enabled? (y/n)");

    let debug_enabled = match term.read_line() {
        Ok(p) => p,
        Err(e) => panic!("{e}"),
    }
    .trim()
    .to_owned()
    .contains("y");

    print!("Disassembly enabled? (y/n)");

    let disasm_enabled = match term.read_line() {
        Ok(p) => p,
        Err(e) => panic!("{e}"),
    }
    .trim()
    .to_owned()
    .contains("y");

    let mut asm = assemble::Assembler::new(&path);
    asm.load();

    let img = match asm.assemble(vec![]) {
        Ok(img) => img,
        Err(errors) => {
            error::AsmblrErr::display(&path, &asm.raw_lines, &errors);

            eprintln!("\n[ASM]\tAssembly failed, {} error(s).", errors.len());
            return;
        }
    };
    asm.vm.debug_enabled = debug_enabled;
    asm.vm.disasm_enabled = disasm_enabled;
    let now = Instant::now();
    asm.link_then_execute(
        &img,
        Some(vec![
            /*"./src/obj_files/flib.asm.obj",*/ "./src/obj_files/stacklib.asm.obj",
        ]),
        Some(vec![putc_x21, puts_x22, getc_x23, halt_x25]),
    );
    let elapsed = now.elapsed();
    println!(
        "\nExecuted {} instructions in {:?}",
        asm.vm.instruction_count, elapsed
    );
    //drop(asm)
    print!("Output to object file? (y/n)");

    let output = match term.read_line() {
        Ok(p) => p,
        Err(e) => panic!("{e}"),
    }
    .trim()
    .to_owned()
    .contains("y");

    if output {
        let _ = match file_io::write_binary_to_file(
            &format!("./src/obj_files/{}.obj", buffer),
            &img,
        ) {
            Ok(size) => println!("[OK]\tWrote {size} bytes to./src/obj_files/{}.obj ", buffer),
            Err(e) => panic!("[FAIL]\t{:?}", e),
        };

        let obj = file_io::read_binary_from_file(
            &format!("./src/obj_files/{}.obj", buffer),
            file_io::Endian::Little,
        )
        .expect("Unable to open multiply.obj");
        for (index, word) in obj.iter().enumerate() {
            print!("{word:04x}");
            if (index + 1) % 8 == 0 {
                print!("\n");
            } else {
                print!(" ");
            }
        }
    } else {
        println!("Skipping..")
    }

    print!("Output to symbol file? (y/n)");

    let output = match term.read_line() {
        Ok(p) => p,
        Err(e) => panic!("{e}"),
    }
    .trim()
    .to_owned()
    .contains("y");

    if output {
        let _ = match file_io::write_symbols_to_file(
            &format!("./src/obj_files/{}.sym", buffer),
            &img,
        ) {
            Ok(size) => println!(
                "[OK]\tWrote {size} symbols to./src/obj_files/{}.obj ",
                buffer
            ),
            Err(e) => panic!("[FAIL]\t{:?}", e),
        };

        let symbols =
            file_io::read_symbols_from_file(&format!("./src/obj_files/{}.sym", buffer)).expect(
                &format!("Unable to load symbols from ./src/obj_files/{}.sym", buffer),
            );
        println!("\n\nSYMBOL FILE \n{:#?}", symbols);
    } else {
        println!("Skipping..")
    }

    return;
}
