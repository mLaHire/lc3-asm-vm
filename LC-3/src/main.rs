use std::io::{self, Read, Stdin};
pub mod binary_utils;
pub mod virtual_machine;

use binary_utils::instructions::get_opcode_4bit;
use virtual_machine as vm;

fn main() {
    let instr: u16 = 0b0110010011000110;
    
    // let mut bit = String::new();
    // io::stdin().read_line(&mut bit).
    //     expect("Unable to read input.");

    // let bit: u32 = match bit.trim().parse() { 
    //     Ok(num) => num,
    //     Err(_) => 0
    // };

   // let instr: u16 = 0b00001011;

    // for n in 0..8{
    //     println!("Bit {n} of '{:04b}' is '{:04b}'", instr, binary_utils::isolate_bit(instr, n.try_into().unwrap()));
    // }

    // println!("INSTRUCTION: {:016x} {:016b} OPCODE: [16-bit] {:016b} [4-bit] {:04b}", instr, instr, instructions::get_opcode_16bit(instr), instructions::get_opcode_4bit(instr));
    // println!("{:06b} sign-extended to 16 bits = {:016b}", 0b011001, binary_utils::sign_extend(0b011001, 5));
    // println!("{:06b} sign-extended to 16 bits = {:016b}", 0b111001, binary_utils::sign_extend(0b111001, 5));

    let a = 0b0000000000001;//binary_utils::sign_extend(0b011001, 5);
    let b = 0b0000000001001;//binary_utils::sign_extend(0b011011, 5);

   // println!(" {:016b} + {:016b} = {:016b} ", a,b,  add_2s_complement(a, b));


    let mut machine = vm::VirtualMachine::new();
    let instr = 0b0001_010_001_1_10111;
    machine.ld_instr_to_mem(instr, vm::PC_START_IDX);
    let and_instr = 0b0101_011_010_0_00010;
    machine.ld_instr_to_mem(and_instr, vm::PC_START_IDX+1);

    println!("{}", get_opcode_4bit(instr));

    println!("\n{:?}", machine.registers);

    machine.fetch();
    machine.decode();
    machine.execute();
    //println!("R2: {:016b}", machine.read_reg(2) );

    println!("\n{:?}", machine.registers);

    machine.fetch();
    machine.decode();
    machine.execute();
    //println!("R3: {:016b}", machine.read_reg(3) );
    println!("\n{:?}", machine.registers);

    let branch_z = 0b0000_010_000000000;
    let branch = branch_z + 0b1_1111_1101;
    
    machine.ld_instr_to_mem(branch, vm::PC_START_IDX+2);

    let branch_nzp = 0b0000_111_000000000;
    let branch = branch_nzp + /*0b1_1111_1110*/  0;
    machine.ld_instr_to_mem(branch, vm::PC_START_IDX+3);

    let st_5 = 0b0011_000_0_0000_1000;
    machine.set_reg(0, 5);
    machine.ld_instr_to_mem(st_5, vm::PC_START_IDX+4);

    let ld =   0b0010_011_0_0000_0111;
    machine.ld_instr_to_mem(ld, vm::PC_START_IDX+5);
    

    loop{
        machine.fetch();
        machine.decode();
        machine.execute();
        println!("{:?} >>", machine.registers);
        let mut k = String::new();

       
        io::stdin().read_line(&mut k).expect("Unable to read input.");
        
    
    }
}

