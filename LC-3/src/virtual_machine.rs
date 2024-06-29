use core::panic;

use crate::binary_utils::{
    self, add_2s_complement, flag_is_set,
    instructions::*,
    is_negative, MAX_MEMORY, MAX_MEMORY_SIZE,
};

pub const PC_START: u16 = 0x300;
pub const PC_START_IDX: usize = 0x300;

#[derive(Debug, PartialEq)]
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
        match n {
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
        }
    }
}

#[derive(Debug)]
pub enum OP {
    BR = 0, // branch
    ADD,    // add
    LD,     // load
    ST,     // store
    JSR,    // jump register
    AND,    // bitwise and
    LDR,    // load register

    RTI, // (unused)

    NOT, // bitwise not
    LDI, // load indirect
    STI, // store indirect
    JMP, // jump

    RES, // reserved (unused)

    LEA,  // load effective address
    TRAP, // execute trap
}

/*pub struct OperationError{
    err: u16,
}*/

#[derive(Debug)]
pub struct VirtualMachine {
    memory: [u16; 0xFFFF],
    pub registers: Registers,

    //r: [u16; 8],
    //condition: ConditionCodes,
    program_counter: u16,
    instruction: Instruction,
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

            instruction: Instruction {
                opcode: OP::RES,
                word: 0,
            },

            registers: Registers::new(),

            program_counter: PC_START,
        }
    }

    fn read_memory(&self, address: u16) -> u16 {
        if address >= MAX_MEMORY {
            panic!("Cannot access memory out of bounds");
        }

        let address: usize = address
            .try_into()
            .expect("Unable to convert from u16 to usize to access memmory.");

        self.memory[address]
    }

    pub fn fetch(&mut self) {
        if self.program_counter > MAX_MEMORY {
            panic!(
                "program_counter {0:04x} ({0:016b} {0}) is out of range",
                self.program_counter
            );
        }

        self.instruction.word = self.memory[self.program_counter as usize];
        let opcode = VirtualMachine::u16_to_opcode(binary_utils::instructions::get_opcode_4bit(
            self.instruction.word,
        ));

        println!(
            "[0x{:04x} @{:05}] {opcode:?} {:016b}",
            self.program_counter as usize, self.program_counter, self.instruction.word
        );
        self.program_counter += 1;
    }

    pub fn decode(&mut self) {
        self.instruction.opcode = VirtualMachine::u16_to_opcode(
            binary_utils::instructions::get_opcode_4bit(self.instruction.word),
        );
    }

    pub fn execute(&mut self) {
        //println!("{:?}", self.instruction.opcode);
        match self.instruction.opcode {
            OP::ADD => self.execute_op_add(),
            OP::AND => self.execute_op_and(),
            OP::BR => self.execute_op_br(),
            OP::ST => self.execute_op_st(),
            OP::LD => self.execute_op_ld(),
            _ => panic!("No valid instruction."),
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

            8 => OP::NOT,
            9 => OP::LDI,
            10 => OP::STI,

            11 => OP::JMP,

            13 => OP::LEA,
            14 => OP::TRAP,

            _ => {
                panic!("Invalid instruction.");
            }
        }
    }
}

impl VirtualMachine {
    pub fn dump_machine(&self) {}

    fn execute_op_add(&mut self) {
        let instr = self.instruction.word;

        let dest_r1 = get_register_at(instr, (9, 11));
        let src_r1 = get_register_at(instr, (6, 8));

        let result: u16;

        if !flag_is_set(instr, 5) {
            let src_r2 = get_register_at(instr, (0, 2));
            result = binary_utils::add_2s_complement(self.read_reg(src_r1), self.read_reg(src_r2));
        } else {
            let imm5_value = get_sign_ext_value(instr, 5);
            result = add_2s_complement(self.read_reg(src_r1), imm5_value);
            println!("{} + {} = {}", self.read_reg(src_r1), imm5_value, result);
        }

        self.set_reg(dest_r1, result);
        self.update_condition(result);
    }

    fn execute_op_and(&mut self) {
        let instr = self.instruction.word;

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

    fn execute_op_br(&mut self) {
        let instr = self.instruction.word;

        let n = flag_is_set(instr, 11);
        let z = flag_is_set(instr, 10);
        let p = flag_is_set(instr, 9);

        let pc_offset_9_ext = get_sign_ext_value(instr, 9);

        if ((self.registers.condition == ConditionCode::NEGATIVE) && n)
            || (self.registers.condition == ConditionCode::ZERO && z)
            || (self.registers.condition == ConditionCode::POSITIVE && p)
        {
            //dbg!(self.program_counter, pc_offset_9_ext, add_2s_complement(self.program_counter, pc_offset_9_ext));
            self.program_counter = add_2s_complement(self.program_counter, pc_offset_9_ext);
        }
    }

    //ST (SR) -> PC+se(pc_offset_9)
    fn execute_op_st(&mut self) {
        let instr = self.instruction.word;

        let src_r1 = get_register_at(instr, (9, 11));
        let pc_offset_9_ext = get_sign_ext_value(instr, 9);

        let evaluated_address: usize = (self.program_counter + pc_offset_9_ext)
            .try_into()
            .expect("Unable to convert from u16 to usize to access memmory.");

        self.memory[evaluated_address] = self.read_reg(src_r1);

        dbg!(evaluated_address);
        dbg!(self.memory[evaluated_address]);
    }

    fn execute_op_ld(&mut self) {
        let instr = self.instruction.word;

        let dest_reg = get_register_at(instr, (9, 11));

        let pc_offset_9_ext = get_sign_ext_value(instr, 9);

        dbg!(self.program_counter + pc_offset_9_ext);
        dbg!(self.read_memory(self.program_counter + pc_offset_9_ext));
        self.set_reg(
            dest_reg,
            self.read_memory(self.program_counter + pc_offset_9_ext),
        );
    }

    pub fn set_reg(&mut self, register: u16, value: u16) {
        self.registers.set(
            match register {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 3,
                5 => 5,
                6 => 6,
                7 => 7,
                _ => {
                    panic!("Invalid write_register R{register}.")
                }
            },
            value,
        );
    }

    pub fn update_condition(&mut self, result: u16) {
        if result == 0 {
            self.registers.condition = ConditionCode::ZERO;
        } else if is_negative(result) {
            self.registers.condition = ConditionCode::NEGATIVE;
        } else {
            self.registers.condition = ConditionCode::POSITIVE;
        }
    }


    pub fn read_reg(&mut self, register: u16) -> u16 {
        self.registers.read(match register {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 3,
            5 => 5,
            6 => 6,
            7 => 7,
            _ => {
                panic!("Invalid read_register R{register}.")
            }
        })
    }
}
