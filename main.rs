use console::Term;
pub mod assemble;
pub mod binary_utils;
pub mod error;
pub mod load_binary;
pub mod virtual_machine;

fn main() {
    let mut asm = assemble::Assembler::new(".\\src\\asm-files\\hello_world.asm");
    asm.load();
    asm.tokenize();

    match asm.parse_origin_and_end() {
        Err(e) => eprintln!("Error finding program .ORIG and .END: {e}"),
        Ok(r) => println!("Program\t.ORIG {:x}\t.END{:x}", r.0, r.1),
    }

    asm.parse_directives();
    asm.parse_instructions();

    return;
}
