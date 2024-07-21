use console::Term;
pub mod assemble;
pub mod binary_utils;
pub mod error;
pub mod load_binary;
pub mod virtual_machine;

fn main() {
    let putc_x21 = assemble::TrapInstruction::new("putc", 0x21);
    let puts_x22 = assemble::TrapInstruction::new("puts", 0x22);
    let getc_x23 = assemble::TrapInstruction::new("getc", 0x23);

    print!("Enter local file path: .\\src\\asm-files\\");
    let mut buffer = match Term::stdout().read_line() {
        Ok(p) => p,
        Err(e) => panic!("{e}"),
    }
    .trim()
    .to_owned();

    let mut path = String::from(".\\src\\asm-files\\");
    path.push_str(&buffer);

    let mut asm = assemble::Assembler::new(&path);
    asm.load();
    asm.tokenize();
    match asm.parse_origin_and_end() {
        Err(e) => eprintln!("Error finding program .ORIG and .END: {e}"),
        Ok(r) => println!("Program\t.ORIG {:x}\t.END{:x}", r.0, r.1),
    }
    asm.load_symbols();

    asm.parse_directives();
    asm.adjust_symbols();
    asm.parse_instructions_then_run(Some(vec![putc_x21, puts_x22, getc_x23]));

    return;
}
