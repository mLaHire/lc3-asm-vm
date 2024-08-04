#[derive(Debug)]
pub enum FileLoadError {
    FsOpenFailed,
    FsReadFailed,
    InvalidBinary,
}

#[derive(Clone, Debug)]
pub struct AsmblrErr{
    pub line_number: u16,
    pub msg: String,
}

impl AsmblrErr{
    pub fn new(line_number: u16, message: String) -> Self{
        AsmblrErr{
            line_number,
            msg: message,
        }
    }

    pub fn display(file_path: &str, raw_lines: &Vec<String>, errors: &Vec<Self>){
        for e in errors{
            // let msg = &e.msg;
            // let line = e.line_number;
            // let line_text = &raw_lines[e.line_number as usize - 1];
            // eprintln!("\nSyntax error:{msg}");
            // eprintln!("in\t({file_path}");
            // eprintln!("    |");
            // eprintln!("{line:4}|\t{line_text}");
            // eprintln!("    |");
           
            let source_text = if e.line_number > 0{
                raw_lines[e.line_number as usize - 1].clone()
            } else{ 
                format!("[Unable to resolve source line, invalid line number ({}) ]", e.line_number)
            };


            eprintln!(
                "\nSyntax error ('{}' (line {})):\n\n{:02}|{}\n\n\t{}",
                file_path, e.line_number, e.line_number, source_text , e.msg
            );
        }
    }
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
