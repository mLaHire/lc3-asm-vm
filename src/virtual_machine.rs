use crate::Term;
use core::panic;
use core::time;
use std::borrow::BorrowMut;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self};
use std::time::Duration;

use crate::binary_utils::{
    self, add_2s_complement, flag_is_set, instructions::*, is_negative, MAX_MEMORY, MAX_MEMORY_SIZE,
};

pub const PC_START: u16 = 0x00;
pub const PC_START_IDX: usize = 0x300;

#[derive(Debug, PartialEq, Clone)]
pub enum ConditionCode {
    POSITIVE = 1 << 0,
    NEGATIVE = 1 << 1,
    ZERO = 1 << 2,
}

#[derive(Debug)]
pub struct Registers {
    r: [u16; 8],
    condition: ConditionCode,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            r: [0; 8],
            condition: ConditionCode::POSITIVE,
        }
    }
    pub fn set(&mut self, n: u16, value: u16) {
        //println!("R{n} <- {value}");
        match n {
            0 => self.r[0] = value,
            1 => self.r[1] = value,
            2 => self.r[2] = value,
            3 => self.r[3] = value,
            4 => self.r[4] = value,
            5 => self.r[5] = value,
            6 => self.r[6] = value,
            7 => self.r[7] = value,
            _ => {
                panic!("Invalid write_register R{n}.")
            }
        }
    }

    pub fn read(&mut self, n: u16) -> u16 {
        let val = match n {
            0 => self.r[0],
            1 => self.r[1],
            2 => self.r[2],
            3 => self.r[3],
            4 => self.r[4],
            5 => self.r[5],
            6 => self.r[6],
            7 => self.r[7],

            _ => {
                panic!("Invalid read_register 'R{n}.'");
            }
        };

        //println!("R{n} = {val}");
        return val;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OP {
    BR = 0, // branch
    ADD,    // add
    LD,     // load
    ST,     // store
    JSR,    // jump register
    AND,    // bitwise and
    LDR,    // load register

    STR,

    RTI, // (unused)

    NOT, // bitwise not
    LDI, // load indirect
    STI, // store indirect
    JMP, // jump

    RES, // reserved (unused)

    LEA,  // load effective address
    TRAP, // execute trap
}

#[derive(Debug)]
pub struct IORegister {
    signal: u16,
    data: u16,
}

#[derive(Debug)]
pub struct VirtualMachine {
    memory: [u16; 0xFFFF],
    pub registers: Registers,

    pub kbsr_address: u16,
    pub kbdr_address: u16,

    pub dsr_address: u16,
    pub ddr_address: u16,

    keyboard_reg_mutex: Arc<Mutex<IORegister>>,
    display_reg_mutex: Arc<Mutex<IORegister>>,

    pub run: bool,
    pub debug_enabled: bool,

    program_counter: u16,
    origin: u16,

    current_instruction: Instruction,
    pub instruction_count: i32,
}

#[derive(Debug)]
pub struct Instruction {
    opcode: OP,
    word: u16,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            memory: [0; binary_utils::MAX_MEMORY_SIZE],

            current_instruction: Instruction {
                opcode: OP::RES,
                word: 0,
            },

            kbsr_address: 0xfe00,
            kbdr_address: 0xfe02,

            dsr_address: 0xfe04,
            ddr_address: 0xfe06,

            keyboard_reg_mutex: Arc::new(Mutex::new(IORegister { data: 0, signal: 0 })),
            display_reg_mutex: Arc::new(Mutex::new(IORegister {
                data: 0,
                signal: 0x8000,
            })),

            registers: Registers::new(),

            run: true,
            debug_enabled: true,

            program_counter: PC_START,
            origin: 0,

            instruction_count: 0,
        }
    }

    pub fn set_program_origin(&mut self, pc: u16) {
        self.program_counter = pc;
        self.origin = pc;
    }

    pub fn run_io_thread(&mut self) {
        self.memory[self.dsr_address as usize] = 0x8000;

        let kb_registers = self.keyboard_reg_mutex.clone();
        let disp_registers = self.display_reg_mutex.clone();
        println!("[IO]\tStarting IO threads...");
        thread::spawn(move || {
            let mut term = Term::stdout();
            //println!("[IO]\tStarting output server.");
            let mut count = 0;
            loop {
                let mut display = loop {
                    let lock_attempt = disp_registers.try_lock();
                    match lock_attempt {
                        Err(e) => {
                            count += 1;
                            if count > 10 {
                                println!("[IO] Waiting for display_reg_mutex. {e}");
                            }
                            thread::sleep(time::Duration::from_millis(5 + count));
                            continue;
                        }

                        Ok(lock) => {
                            count = 0;
                            break lock;
                        }
                    }
                };
                //println!("[IO] DSR={:016b}", display.signal);
                if (*display).signal == 0x0000
                /*&& (*display).data != 0*/
                {
                    //print!("[IO] [DISP] Output detected {}", display.data);
                    //There is a char to read from DDR which has not been displayed yet.
                    //(*display).signal[15 = 1;
                    if display.data > 127 {
                        let ch: String = String::from_utf16_lossy(&[display.data]);
                        drop(display);
                        panic!("[IO ERROR] Non-ascii character '{}' in DDR.", ch);
                    }
                    //println!("[IO]\tDisplaying {:08b} ASCII {}", binary_utils::truncate_to_bit((*display).data, 7), binary_utils::truncate_to_bit((*display).data, 7));

                    match term.write(&[binary_utils::truncate_to_bit((*display).data, 7) as u8]) {
                        Ok(_) => {
                            (*display).signal = 0x8000;
                        }
                        Err(e) => {
                            println!("[IO] Error writing to console {e:?}");
                        }
                    }
                }
                drop(display);
                thread::sleep(time::Duration::from_millis(1));
            }
        });

        let input_server = thread::spawn(move || {
            let term = Term::stdout();
            //println!("[IO]\tStarting input server.");
            loop {
                let input_char: u16 = term
                    .read_char()
                    .expect("Failure to read input char.")
                    .try_into()
                    .expect("msg");

                // println!(
                //     "INPUT: '{}' (ASCII {input_char})",
                //     char::from_u32(input_char as u32).unwrap()
                // );

                let mut attempt_count = 0;
                let mut kb_regs = loop {
                    let lock_attempt = kb_registers.try_lock();
                    match lock_attempt {
                        Err(e) => {
                            // if attempt_count > 10 {
                            //     println!("[IO] Waiting for keyboard_register_mutex. {e}");

                            // }
                            // attempt_count += 1;
                            // thread::sleep(time::Duration::from_millis(5 + attempt_count));
                            continue;
                        }

                        Ok(lock) => {
                            break lock;
                        }
                    }
                };

                if !binary_utils::flag_is_set((*kb_regs).signal, 15) {
                    //println!("[IO]\tChar input recived. KBSR is clear. Updating KBSR and KBDR");
                    (*kb_regs).signal = binary_utils::set_flag_true(0, 15);
                    (*kb_regs).data = input_char;
                } else {
                    //println!("[IO]\tChar input recived. KBSR is NOT clear. Ignoring input.");
                }
                drop(kb_regs);
                //println!("[IO] Dropped kb_regs.");
                thread::sleep(time::Duration::from_millis(2));
            }
        });
    }

    pub fn load_binary_into_memory(&mut self, binary: Vec<u16>, program_start_addr: u16) {
        if binary.is_empty() {
            panic!("Attempt to load empty binary into memory.");
        }

        let mut offset = 0;
        for instr in binary {
            self.write_memory(program_start_addr + offset, instr);
            offset += 1;
        }

        println!(
            "Loaded binary ({} instructions) into memory, from ADDRESS {:x} to {:x}. ",
            offset,
            program_start_addr,
            program_start_addr + offset
        );
    }

    pub fn read_memory(&mut self, address: u16) -> u16 {
        if address >= MAX_MEMORY {
            panic!("RUNTIME ERROR: Cannot access memory out of bounds.");
        }

        if address == self.kbsr_address {
            if self.debug_enabled {
                println!("\t\t(!)\tReading Keyboard Registers (SIGNAL)...(!)\t");
            }

            // let kb_reg = self
            //     .keyboard_reg_mutex
            //     .try_lock();
            //     .expect("[CPU]\t[KBSR READ] Failed to lock Keyboard Registers.");
            let mut attempt_count = 0;
            let kb_reg = loop {
                let lock_attempt = self.keyboard_reg_mutex.try_lock();
                match lock_attempt {
                    Err(e) => {
                        if attempt_count > 10 {
                            println!("[CPU] Waiting for keyboard_register_mutex. {e}");
                        }
                        attempt_count += 1;
                        thread::sleep(time::Duration::from_millis(5 + attempt_count));
                        continue;
                    }

                    Ok(lock) => {
                        break lock;
                    }
                }
            };
            let signal = kb_reg.signal;
            self.memory[address as usize] = signal;
            drop(kb_reg);
            thread::sleep(time::Duration::from_millis(10));
        }

        if address == self.kbdr_address {
            if self.debug_enabled {
                println!("[CPU]\t(!)Reading kbdr\t\t(!)");
            }

            let mut kb_reg = self
                .keyboard_reg_mutex
                .try_lock()
                .expect("[CPU] Failed to lock kb_reg (DATA)");
            // if binary_utils::flag_is_set(kb_reg.signal, 15){

            // }
            self.memory[address as usize] = kb_reg.data;
            kb_reg.signal = 0;
            drop(kb_reg);
            //thread::sleep(time::Duration::from_millis(10));
        }

        if address == self.dsr_address {
            /*let disp_reg = self
            .display_reg_mutex
            .try_lock()
            .expect("[CPU] Failed to lock disp_reg (signal)");*/
            //let mut attempt_count = 0;
            let disp_reg = loop {
                let lock_attempt = self.display_reg_mutex.try_lock();

                match lock_attempt {
                    Err(e) => {
                        // attempt_count += 1;
                        // if attempt_count > 10 {
                        //     //println!("[CPU] [READ] Waiting for display_reg_mutex. {e}");
                        // }

                        //thread::sleep(time::Duration::from_millis(1 + attempt_count));
                        continue;
                    }

                    Ok(lock) => {
                        break lock;
                    }
                }
            };
            self.memory[address as usize] = disp_reg.signal;
            if self.debug_enabled {
                println!("[CPU] Reading DSR. {}", self.memory[address as usize]);
            }
        }

        if address == self.ddr_address {
            let disp_reg = self
                .keyboard_reg_mutex
                .try_lock()
                .expect("[CPU] Failed to lock disp_reg (DATA)");

            // if binary_utils::flag_is_set(kb_reg.signal, 15){

            // }
            self.memory[address as usize] = disp_reg.data;
        }

        let address: usize = address
            .try_into()
            .expect("Unable to convert from u16 to usize to access memmory.");

        if self.debug_enabled {
            println!(
                "[READ]\tMEM[{address:04x}] (={})\t",
                binary_utils::as_negative_i32(self.memory[address])
            );
        }
        self.memory[address]
    }

    pub fn write_memory(&mut self, address: u16, value: u16) {
        if address >= MAX_MEMORY {
            panic!("[WRITE]\tCannot access memory out of bounds/");
        }

        if address == self.kbsr_address {
            // let mut kbsr_updater = self.kbsr_mutex.lock().unwrap();
            // *kbsr_updater = value;

            let mut kb_reg = self
                .keyboard_reg_mutex
                .try_lock()
                .expect("[CPU][WRITE MEM]\tUnable to lock kb_reg.");
            (*kb_reg).data = value;
        }

        if address == self.ddr_address {
            let mut attempt_count = 0;
            let mut disp_reg = loop {
                let lock_attempt = self.display_reg_mutex.try_lock();
                match lock_attempt {
                    Err(e) => {
                        if attempt_count > 10 {
                            //thread::sleep(time::Duration::from_millis(attempt_count-5));
                            //println!("[CPU] [WRITE] Waiting for display_reg_mutex. {e}");
                        }
                        attempt_count += 1;

                        continue;
                    }

                    Ok(lock) => {
                        break lock;
                    }
                }
            };
            (*disp_reg).data = value;
            (*disp_reg).signal = 0; //Clear DSR; not ready for another char
            drop(disp_reg);
        }

        let address: usize = address
            .try_into()
            .expect("Unable to convert from u16 to usize to access memmory.");

        self.memory[address] = value;
        if self.debug_enabled {
            println!(
                "[WRITE]\t0x{value:04x}\t--> MEM[0x{:04x}   #{address:06}]",
                address
            );
        };
    }

    pub fn fetch(&mut self) {
        if self.program_counter >= MAX_MEMORY {
            panic!(
                "program_counter {0:04x} ({0:016b} {0}) is out of range",
                self.program_counter
            );

            //return;
        }

        self.current_instruction.word = self.memory[self.program_counter as usize];
        let opcode = VirtualMachine::u16_to_opcode(binary_utils::instructions::get_opcode_4bit(
            self.current_instruction.word,
        ));

        if self.debug_enabled {
            println!(
                //@{:05}
                "[0x{:04x}\t0+0x{:03x}]\t\t{opcode:?}\t{:016b}",
                self.program_counter as usize,
                (self.program_counter as i32 - self.origin as i32),
                /*self.program_counter,*/ self.current_instruction.word
            );
        }
        self.program_counter += 1;
    }

    pub fn decode(&mut self) {
        self.current_instruction.opcode = VirtualMachine::u16_to_opcode(
            binary_utils::instructions::get_opcode_4bit(self.current_instruction.word),
        );
    }

    pub fn execute(&mut self, symbol_table: Option<&crate::assemble::SymbolTable>) {
        //println!("{:?}", self.instruction.opcode);
        let instr = self.current_instruction.word;
        let disassem = crate::assemble::Parser::dissasemble_memory(
            instr,
            Some(self.program_counter),
            symbol_table,
            None,
        );
        println!("0x{:04x}\t{instr:016b}\t\t{disassem}", self.program_counter-1);
        match self.current_instruction.opcode {
            OP::ADD => self.execute_op_add(instr),
            OP::AND => self.execute_op_and(instr),
            OP::BR => self.execute_op_br(instr),
            OP::ST => self.execute_op_st(instr),
            OP::STI => self.execute_op_sti(instr),
            OP::LD => self.execute_op_ld(instr),
            OP::NOT => self.execute_op_not(instr),
            OP::LDI => self.execute_op_ldi(instr),
            OP::TRAP => self.execute_op_trap(instr),
            OP::JMP => self.execute_op_jmp(instr),
            OP::LEA => self.execute_op_lea(instr),
            OP::LDR => self.execute_op_ldr(instr),
            OP::STR => self.execute_op_str(instr),
            OP::JSR => self.execute_op_jsr(instr),
            OP::RES => self.run = false,
            _ => panic!("No valid instruction."),
        }
        self.instruction_count += 1;
        if self.debug_enabled {
            for i in 0..=7 {
                print!(
                    "|R{i}: {:05}| ",
                    binary_utils::as_negative_i32(self.read_reg(i))
                );
            }
            print!("\n");
        }
    }

    pub fn ld_instr_to_mem(&mut self, instr: u16, addr: usize) {
        if addr > MAX_MEMORY_SIZE {
            panic!();
        }

        self.memory[addr] = instr;
    }

    fn u16_to_opcode(opcode_4bit: u16) -> OP {
        match opcode_4bit {
            0 => OP::BR,
            1 => OP::ADD,
            2 => OP::LD,
            3 => OP::ST,
            4 => OP::JSR,
            5 => OP::AND,
            6 => OP::LDR,
            7 => OP::STR,

            8 => OP::STI,

            9 => OP::NOT,
            10 => OP::LDI,
            11 => OP::STI,

            12 => OP::JMP,

            14 => OP::LEA,
            15 => OP::TRAP,

            _ => {
                //println!("\n[CPU]\tHALT");
                OP::RES
                //panic!("Invalid instruction. {opcode_4bit:04b}");
            }
        }
    }
}

//Execute instruction
impl VirtualMachine {
    pub fn dump_machine(&self) {}

    fn execute_op_add(&mut self, instr: u16) {
        let dest_r1 = get_register_at(instr, (9, 11));
        let src_r1 = get_register_at(instr, (6, 8));

        let result: u16;

        if !flag_is_set(instr, 5) {
            let src_r2 = get_register_at(instr, (0, 2));
            result = binary_utils::add_2s_complement(self.read_reg(src_r1), self.read_reg(src_r2));
            // println!("R{dest_r1} <- {} + {} = {}", binary_utils::as_negative_i32(self.read_reg(src_r1)), binary_utils::as_negative_i32(self.read_reg(src_r2)), binary_utils::as_negative_i32(result));
        } else {
            let imm5_value = get_sign_ext_value(instr, 5);
            result = add_2s_complement(self.read_reg(src_r1), imm5_value);
            //println!("Src '{}' + Imm5 '{:05b}' = {}", self.read_reg(src_r1), binary_utils::as_negative_i16(imm5_value), binary_utils::as_negative_i16(result));
        }

        self.set_reg(dest_r1, result);
        self.update_condition(result);
    }

    fn execute_op_and(&mut self, instr: u16) {
        let dest_r1 = get_register_at(instr, (9, 11));
        let src_r1 = get_register_at(instr, (6, 8));

        let result: u16;

        if !flag_is_set(instr, 5) {
            let src_r2 = get_register_at(instr, (0, 2));

            result = self.read_reg(src_r1) & self.read_reg(src_r2);
            /*println!(
                "AND R{dest_r1} <- (R{src_r1}, R{src_r2}) \t {0:b} & {1:b} = {result:b}",
                self.read_reg(src_r1),
                self.read_reg(src_r2)
            );*/
        } else {
            let imm_val_5 = get_sign_ext_value(instr, 5);
            result = self.read_reg(src_r1) & imm_val_5;
        }

        self.update_condition(result);
        self.set_reg(dest_r1, result);
    }

    //NOT DR, SR
    fn execute_op_not(&mut self, instr: u16) {
        let dest_reg = get_register_at(instr, (9, 11));
        let src_reg = get_register_at(instr, (6, 8));

        let result = !self.read_reg(src_reg);

        self.update_condition(result);
        self.set_reg(dest_reg, result);
    }

    fn execute_op_br(&mut self, instr: u16) {
        let n = flag_is_set(instr, 11);
        let z = flag_is_set(instr, 10);
        let p = flag_is_set(instr, 9);

        let pc_offset_9_ext = get_sign_ext_value(instr, 9);

        if ((self.registers.condition == ConditionCode::NEGATIVE) && n)
            || (self.registers.condition == ConditionCode::ZERO && z)
            || (self.registers.condition == ConditionCode::POSITIVE && p)
        {
            //dbg!(self.program_counter, pc_offset_9_ext, add_2s_complement(self.program_counter, pc_offset_9_ext));
            //println!("PC_OFFSET_9 = {}", binary_utils::as_negative_i16(pc_offset_9_ext));
            self.program_counter = add_2s_complement(self.program_counter, pc_offset_9_ext);
            //println!("Branching to instruction {:04x}", self.program_counter);
        } else {
            if self.debug_enabled {
                println!(
                    "\n[CPU]\t\tNot branching, continuing to 0x{:04x}",
                    self.program_counter + 1
                );
            }
        }
    }

    //ST (SR) -> PC+se(pc_offset_9)
    fn execute_op_st(&mut self, instr: u16) {
        let src_r1 = get_register_at(instr, (9, 11));
        let source_reg_contents = self.read_reg(src_r1);

        let pc_offset_9_ext = get_sign_ext_value(instr, 9);

        let evaluated_address = (self.program_counter + pc_offset_9_ext)
            .try_into()
            .expect("Unable to convert from u16 to usize to access memmory.");

        self.write_memory(evaluated_address, source_reg_contents);

        /*dbg!(evaluated_address);
        dbg!(self.memory[evaluated_address as usize]);*/
    }

    fn execute_op_sti(&mut self, instr: u16) {
        let src_reg = get_register_at(instr, (9, 11));
        let src_reg_contents = self.read_reg(src_reg);

        let pc_offset_9_ext = get_sign_ext_value(instr, 9);

        let evaluated_ptr_address: usize = (self.program_counter + pc_offset_9_ext)
            .try_into()
            .expect("Unable to convert from u16 to usize to access memmory.");

        let final_target_addr = self.memory[evaluated_ptr_address];

        self.write_memory(final_target_addr, src_reg_contents);
    }

    fn execute_op_ld(&mut self, instr: u16) {
        let dest_reg = get_register_at(instr, (9, 11));

        let pc_offset_9_ext = get_sign_ext_value(instr, 9);

        /*dbg!(self.program_counter + pc_offset_9_ext);
        dbg!(self.read_memory(self.program_counter + pc_offset_9_ext));*/
        let address = self.program_counter + pc_offset_9_ext;
        let value = self.read_memory(address);
        self.set_reg(dest_reg, value);
        self.update_condition(value);
    }

    fn execute_op_ldr(&mut self, instr: u16) {
        let dest_reg = get_register_at(instr, (9, 11));
        let base_reg = get_register_at(instr, (6, 8));

        let offset_6_ext = get_sign_ext_value(instr, 6);

        /*dbg!(self.program_counter + pc_offset_9_ext);
        dbg!(self.read_memory(self.program_counter + pc_offset_9_ext));*/
        // dbg!(offset_6_ext);
        let address = binary_utils::add_2s_complement(self.read_reg(base_reg), offset_6_ext);
        let value = self.read_memory(address);
        self.update_condition(value);
        self.set_reg(dest_reg, value);
    }

    fn execute_op_str(&mut self, instr: u16) {
        let src_reg = get_register_at(instr, (9, 11));
        let base_reg = get_register_at(instr, (6, 8));

        let offset_6_ext = get_sign_ext_value(instr, 6);

        /*dbg!(self.program_counter + pc_offset_9_ext);
        dbg!(self.read_memory(self.program_counter + pc_offset_9_ext));*/
        // dbg!(offset_6_ext);
        let address = binary_utils::add_2s_complement(self.read_reg(base_reg), offset_6_ext);
        let src_reg_value = self.read_reg(src_reg);
        self.write_memory(address, src_reg_value);
        // let value = self.read_memory(address);
        // self.update_condition(value);
        // self.set_reg(dest_reg, value);
    }

    fn execute_op_lea(&mut self, instr: u16) {
        let dest_reg = get_register_at(instr, (9, 11));

        let pc_offset_9_ext = get_sign_ext_value(instr, 9);

        /*dbg!(self.program_counter + pc_offset_9_ext);
        dbg!(self.read_memory(self.program_counter + pc_offset_9_ext));*/
        let address = binary_utils::add_2s_complement(self.program_counter, pc_offset_9_ext);
        self.update_condition(address);
        self.set_reg(dest_reg, address);
    }

    fn execute_op_ldi(&mut self, instr: u16) {
        let dest_reg = get_register_at(instr, (9, 11));

        let pc_offset_9_sext = get_sign_ext_value(instr, 9);

        /*dbg!(self.program_counter + pc_offset_9_ext);
        dbg!(self.read_memory(self.program_counter + pc_offset_9_ext));*/

        let address_to_load_from = self.read_memory(self.program_counter + pc_offset_9_sext);
        let value_at_address = self.read_memory(address_to_load_from);

        self.set_reg(dest_reg, value_at_address);

        let result: u16 = self.read_reg(dest_reg);
        self.update_condition(result);
    }

    // JMP BASE-REGISTER. Unconditional jump to addr specificed by R[base]
    fn execute_op_jmp(&mut self, instr: u16) {
        let base_reg = get_register_at(instr, (6, 8));
        let address_at_base_reg = self.read_reg(base_reg);

        self.program_counter = address_at_base_reg;
    }

    fn execute_op_trap(&mut self, instr: u16) {
        let trap_vector = binary_utils::truncate_to_n_bit(instr, 8);
        if !trap_vector <= 0xff {
            panic!("TRAP vector 0x{trap_vector:04X} must be <= 0x00FF");
        }

        let address_of_subroutine = self.read_memory(trap_vector);
        if self.debug_enabled {
            println!("\n[CPU]\t[TRAP]\tEXECUTING TRAP (0x{trap_vector:x}) Called at 0x{:x} ---> 0x{address_of_subroutine:x}\n", self.program_counter - 1);
        }
        self.set_reg(7, self.program_counter);
        self.program_counter = address_of_subroutine;
        //let
    }

    //JSR LABEL (Pc offset 11) [instr[11] = 1]
    //JSR(R)    REG             [instr[11] = 0]
    fn execute_op_jsr(&mut self, instr: u16) {
        let subroutine_addr;

        if flag_is_set(instr, 11) {
            //JSR
            let pcoffset11_sext = get_sign_ext_value(instr, 11);
            subroutine_addr = pcoffset11_sext + self.program_counter;
        } else {
            subroutine_addr = get_register_at(instr, (6, 8));
        }

        self.set_reg(7, self.program_counter); //Save addr of next instr
        self.program_counter = subroutine_addr;
    }
    pub fn set_reg(&mut self, register: u16, value: u16) {
        if register > 7 {
            panic!("Invalid write_register R{register}.")
        }
        self.registers.set(register, value);
    }

    pub fn update_condition(&mut self, result: u16) {
        let intial = if self.debug_enabled {
            self.registers.condition.clone()
        } else {
            ConditionCode::ZERO
        };

        if result == 0 {
            self.registers.condition = ConditionCode::ZERO;
        } else if is_negative(result) {
            self.registers.condition = ConditionCode::NEGATIVE;
        } else {
            self.registers.condition = ConditionCode::POSITIVE;
        }
        if self.debug_enabled {
            println!(
                "[CPU][CC]\t\t\t[{}]\t{}",
                match self.registers.condition {
                    ConditionCode::NEGATIVE => "-",
                    ConditionCode::ZERO => "0",
                    ConditionCode::POSITIVE => "+",
                },
                if intial != self.registers.condition {
                    "[CHANGE]\n"
                } else {
                    ""
                }
            );
        }
    }

    pub fn read_reg(&mut self, register: u16) -> u16 {
        if register > 7 {
            panic!("Invalid read_register R{register}.")
        }
        self.registers.read(register)
    }
}
