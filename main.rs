use console::Term;
pub mod assemble;
pub mod binary_utils;
pub mod error;
pub mod load_binary;
pub mod virtual_machine;

struct Trap{
    instructions: Vec<u16>,
    origin: u16,
}



fn main() {
    let output_x21 = assemble::TrapInstruction::new("output", 0x21);

    let mut asm = assemble::Assembler::new(".\\src\\asm-files\\hello_world.asm");
    asm.load();
    asm.tokenize();
    match asm.parse_origin_and_end() {
        Err(e) => eprintln!("Error finding program .ORIG and .END: {e}"),
        Ok(r) => println!("Program\t.ORIG {:x}\t.END{:x}", r.0, r.1),
    }
    asm.load_symbols();

    asm.parse_directives();
    asm.parse_instructions_then_run(Some(vec![output_x21]));

    return;
}
