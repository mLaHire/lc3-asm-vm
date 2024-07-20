use crate::virtual_machine;

#[derive(Debug)]
pub enum FileLoadError {
    FsOpenFailed,
    FsReadFailed,
    InvalidBinary,
}

pub enum AssemblerError {
    FailureToTokenize,
    InvalidString,
}

#[derive(Debug)]
pub enum VirtualMachineErrorType {
    InvalidOpcode,
    InvalidInstruction,
    MachineFailureToExecuteInstruction,
}

//#[derive(Debug)]
// pub struct VirtualMachineError{
//     filename: String,
//     instruction: virtual_machine::Instruction,
//     program_counter: u16,
//     error_type: VirtualMachineErrorType,
//     message: String,
// }

// impl VirtualMachineError{
//     fn new (message:String, filename: String, instruction: virtual_machine::Instruction, program_counter: u16, error_type: VirtualMachineErrorType) -> Self{
//         VirtualMachineError{
//             filename,
//             error_type,
//             instruction,
//             program_counter,
//             message,
//         }
//     }
// }
