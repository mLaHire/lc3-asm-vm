#[derive(Debug)]
pub enum FileLoadError {
    FsOpenFailed,
    FsReadFailed,
    FsWriteFailed,
    InvalidBinary,
    InvalidSymbols,
}

#[derive(Debug)]
pub enum LinkErrorType {
   ResolutionError,
   RangeError
}


#[derive(Clone, Debug)]
pub enum AsmErrorType {
    LinkError,
    SyntaxError,
}

impl AsmErrorType{
    pub fn as_str(&self) -> &str{
        match self{
            Self::LinkError => "Link Error",
            Self::SyntaxError => "Syntax Error",
        }
    }
}


#[derive(Clone, Debug)]
pub struct AsmblrErr {
    pub line_number: Option<u16>,
    pub msg: String,
    pub error_type: AsmErrorType,
}

impl AsmblrErr {
    pub fn new(line_number: Option<u16>, message: String) -> Self {
        AsmblrErr {
            line_number,
            msg: message,
            error_type: AsmErrorType::SyntaxError,
        }
    }

    pub fn link_error(&mut self) -> &mut Self{
        self.error_type = AsmErrorType::LinkError;
        self
    }

    pub fn display(file_path: &str, raw_lines: &Vec<String>, errors: &Vec<Self>) {
        for e in errors {
            // let msg = &e.msg;
            // let line = e.line_number;
            // let line_text = &raw_lines[e.line_number as usize - 1];
            // eprintln!("\nSyntax error:{msg}");
            // eprintln!("in\t({file_path}");
            // eprintln!("    |");
            // eprintln!("{line:4}|\t{line_text}");
            // eprintln!("    |");

            match e.line_number{
                Some(line_number) => {
                    let source_text = if line_number > 0 {
                        raw_lines[line_number as usize - 1].clone()
                        
                    } else {
                        format!(
                            "[Unable to resolve source line, invalid line number ({}) ]",
                            line_number
                        )
                    };

                    eprintln!(
                        "\n{}\t(in '{}' (line {})):\n\n{:02}|{}\n\n\t{}\n",
                        e.error_type.as_str().to_ascii_uppercase(), file_path, line_number, line_number, source_text, e.msg
                    );
                },
                None => {
                    eprintln!(
                        "{}\t(in '{}'):\t{}",
                        e.error_type.as_str().to_ascii_uppercase(), file_path,  e.msg
                    );
                }
            }

            // let source_text = if e.line_number > 0 {
            //     raw_lines[e.line_number as usize - 1].clone()
                
            // } else {
            //     // format!(
            //     //     "[Unable to resolve source line, invalid line number ({}) ]",
            //     //     e.line_number
            //     // )
            //     eprintln!(
            //         "\n{} ('{}' (line {})):\n\n{:02}|{}\n\n\t{}",
            //         e.error_type.as_str().to_ascii_uppercase(), file_path, e.line_number, e.line_number, source_text, e.msg
            //     );
            //     return;
            // };

           

           
        }
    }
}

#[derive(Debug)]
pub enum VirtualMachineErrorType {
    InvalidOpcode,
    InvalidInstruction,
    MachineFailureToExecuteInstruction,
}

// pub enum CliErrorType{
//     InvalidArgs(String),
// }

pub struct CliError{
    message: String,
}

impl CliError{
    pub fn new(message: &str) -> Self{
        CliError{
            message: message.into()
        }
    }

    pub fn dipsplay(&self){
        eprintln!("CLI error: {}", self.message)
    }
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
