use core::time;
use std::{collections::VecDeque, io, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}};
use io::Read;
use std::thread;
use std::sync::mpsc;

use console::Term;

pub mod binary_utils;
pub mod virtual_machine;
pub mod load_binary;
pub mod error;
pub mod assemble;


use virtual_machine::{self as vm, PC_START, PC_START_IDX};

fn main() {
    // let file_contents = assemble::read_asm_file(".\\src\\asm-files\\test2.asm")
    //     .unwrap();

    // for line in file_contents.iter(){
    //     println!("{:03} \t {}", line.number + 1, line.text);
    // }
    // dbg!(assemble::build_symbol_table(&file_contents));


    let mut asm = assemble::Assembler::new(".\\src\\asm-files\\test2.asm");
    asm.load();
    asm.tokenize();

    let sentence = "ADD R0, R1, #-255";
    println!("{:?}", assemble::Token::tokenize_str(sentence));

    let sentence = "ADD R0, R1, x11";
    println!("{:?}", assemble::Token::tokenize_str(sentence));

    let sentence = "LBL ST R1, SAVE1";
    println!("{:?}", assemble::Token::tokenize_str(sentence));


    return;

    let mut machine = vm::VirtualMachine::new();
    // let instr = 0b0001_010_001_1_10111;
    // machine.ld_instr_to_mem(instr, vm::PC_START_IDX);
    // let and_instr = 0b0101_011_010_0_00010;
    // machine.ld_instr_to_mem(and_instr, vm::PC_START_IDX + 1);

    // println!("{}", get_opcode_4bit(instr));

    // println!("\n{:?}", machine.registers);

    // machine.fetch();
    // machine.decode();
    // machine.execute();
    // //println!("R2: {:016b}", machine.read_reg(2) );

    // println!("\n{:?}", machine.registers);

    // machine.fetch();
    // machine.decode();
    // machine.execute();
    // //println!("R3: {:016b}", machine.read_reg(3) );
    // println!("\n{:?}", machine.registers);

    // let branch_z = 0b0000_010_000000000;
    // let branch = branch_z + 0b1_1111_1101;

    // machine.ld_instr_to_mem(branch, vm::PC_START_IDX + 2);

    // let branch_nzp = 0b0000_111_000000000;
    // let branch = branch_nzp + /*0b1_1111_1110*/  0;
    // machine.ld_instr_to_mem(branch, vm::PC_START_IDX + 3);

    // let st_5 = 0b0011_000_0_0000_1000;
    // machine.set_reg(0, 5);
    // machine.ld_instr_to_mem(st_5, vm::PC_START_IDX + 4);

    // let ld = 0b0010_011_0_0000_0111;
    // machine.ld_instr_to_mem(ld, vm::PC_START_IDX + 5);

    // let add = 0b0001_111_000_0_00_011;
    // machine.ld_instr_to_mem(add, vm::PC_START_IDX + 6);

    // let not = 0b1001_111_001_111111;
    // machine.ld_instr_to_mem(not, vm::PC_START_IDX + 7);
    println!("{}", std::env::current_dir().unwrap().display());
    let result = load_binary::read_binary_from_file(String::from("C:\\Users\\Admin\\PROJECTS\\RUST\\LC-3\\src\\test")).expect("...?");

    println!("{:?}", result);

    for i in 0..result.len(){
        println!("{}\t{:04b} {:012b}", i, binary_utils::instructions::get_opcode_4bit(result[i]), binary_utils::isolate_bits_then_shift(result[i], (0,12)));
    }

    // println!("Loaded instructions:");
    // for r in result.iter(){
    //     println!("{:016b}", r);
    // }

    machine.load_binary_into_memory(result, PC_START);
    // for pos in PC_START_IDX..=PC_START_IDX+4{
    //     println!("{:016b}", machine.read_memory(pos as u16));
    // }
   
    


    let mut k = String::new();
    io::stdin()
            .read_line(&mut k)
            .expect("Unable to read input.");

    let (tx, rx) = mpsc::channel::<u16>();

    // let mut char_input_queue = Arc::new(Mutex::new(Vec::<u16>::new()));

    // let char_input_queue2 = Arc::clone(&char_input_queue);

    // let mut is_there_unread_input = Arc::new(AtomicBool::new(false));
    
    // //Arc::new(Mutex::new(true));
    // let mut is_there_unread_input2 = is_there_unread_input.clone();

    thread::spawn(move || {
        let mut queue = VecDeque::<u16>::new();
        
       
        loop{ 

            
            
            let input_char: u16;
            //io::stdin().read_exact(&mut input_char).expect("Error reading char.");
            let term = Term::stdout();
            input_char = term.read_char().expect("Failure to read input char.").try_into().expect("msg");
                  
            
            // if is_there_unread_input2.load(Ordering::Relaxed){
            //         println!("Still waiting to be read.");
            //         is_there_unread_input2.store(true, Ordering::Relaxed);
            //         queue.push_front(input_char);
            // }else{
                tx.send(input_char).expect("Failure to send input char '{input_char}'");
            // }
                
            
            print!("Queue: {queue:?}");
            
            
        }
    });

    const KBSR_LABEL: u16 = vm::PC_START + 100;
    const KBDR_LABEL: u16 = vm::PC_START + 101;

    
    machine.write_memory(KBSR_LABEL, machine.kbsr_addr);
    machine.write_memory(KBDR_LABEL, machine.kbdr_addr);

    // LD R1, #98
    // let ldi = 0b1010_001_0_0000_0000 + 98;
                   
    // machine.ld_instr_to_mem(ldi, vm::PC_START_IDX+1);
    
    // let brzp: u16 = 0b0000_011_1_1111_1110;
    // machine.ld_instr_to_mem(brzp, vm::PC_START_IDX+2);

    // let ldi = 0b1010_000_0_0000_0000 + 97;
    // machine.ld_instr_to_mem(ldi, vm::PC_START_IDX+3);

    // //cloding -> coding that's so essential its like clothing :))

    // machine.ld_instr_to_mem(0b1111_0000_0000_0000, vm::PC_START_IDX+4);


    // for pos in PC_START_IDX..=PC_START_IDX+4{
    //     println!("{:016b}", machine.read_memory(pos as u16));
    // }

    println!("{:016b} {:016b}", 0b0000111_1_0000_0000 - 5, -5);
    // panic!(".");

    loop {
        machine.fetch();
        machine.decode();
        machine.execute();
        // {
        //     let mut unread = is_there_unread_input.lock().unwrap();
        //     if binary_utils::flag_is_set(machine.read_memory(machine.kbsr_addr), 15) != *unread {
        //         println!("updating unread char flag");
        //         *unread = false;
        //     }
        // }
       // println!("{:?} >>", machine.registers);
        println!("KBDR: {:016b} \t KBSR: {:016b}", machine.read_kbdr_wout_update(), machine.read_memory(machine.kbsr_addr));

        match rx.try_recv() {
            Err(_) => {}
            Ok(val) => {
                println!("Char read {}.", val);
              
                machine.update_kbdr(val);
                /*machine.write_memory(machine.kbdr_addr, val);
                machine.write_memory(machine.kbsr_addr, 1 << 15);*/
            }   
        }

        thread::sleep(time::Duration::from_millis(500));
        /*let mut k = String::new();

        io::stdin()
            .read_line(&mut k)
            .expect("Unable to read input.");*/
    }
}
