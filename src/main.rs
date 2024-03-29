#![allow(clippy::let_and_return)]
#![allow(clippy::too_many_arguments)]

mod constants;
mod decoder;
mod tests;

use anyhow::Context;
use execute::Execute;
use std::process::Command;

use std::{
    fs::{self, File},
    io::Write as IoWrite,
    path::PathBuf,
    str::FromStr,
};

pub mod prelude {
    pub use crate::constants::*;
    pub use anyhow::{bail, Result};
}

use decoder::*;
use prelude::*;

//* Outputs the assembled file in the same folder as asm_filepath
fn execute_nasm(asm_filepath: &str) -> Result<()> {
    //*give output to nasm, then open test and compare bytes
    const NASM_PATH: &str = "nasm";

    let mut first_command = Command::new(NASM_PATH);

    first_command.arg(asm_filepath);

    if first_command.execute_check_exit_status_code(0).is_err() {
        bail!(
            "The path `{}` is not a correct nasm executable binary file.",
            NASM_PATH
        );
    }

    Ok(())
}

fn decode_single_instruction(
    decoder: &mut Decoder,
    instructions: &[u8],
    offset: usize,
    outputs: &mut Vec<InstructionWithOffset>,
) -> Result<NumBytesInInstruction> {
    let mut output = String::new();

    let first_byte = instructions[offset];
    let num_bytes_in_instruction =
        decoder.funcs[first_byte as usize](instructions, offset, &mut output, decoder)?;

    outputs.push(InstructionWithOffset { offset, output });

    Ok(num_bytes_in_instruction)
}

fn decode_instructions(instructions: &[u8]) -> Result<Vec<String>> {
    let mut outputs: Vec<InstructionWithOffset> = vec![];

    let mut decoder = Decoder::new();

    let mut bytes_processed = 0;
    while bytes_processed < instructions.len() {
        bytes_processed +=
            decode_single_instruction(&mut decoder, instructions, bytes_processed, &mut outputs)?;
    }

    for ins in &outputs {
        println!("offset: {:x}\noutput: {}", ins.offset, ins.output);
    }

    for ins in decoder.enqued_labels {
        println!("enq_offset: {:x}\nenq_output: {}\n", ins.offset, ins.output);

        if let Ok(index) = outputs.binary_search(&InstructionWithOffset {
            offset: ins.offset,
            output: String::new(),
        }) {
            outputs.insert(
                index,
                InstructionWithOffset {
                    offset: ins.offset,
                    output: format!("{}:\n", ins.output),
                },
            );
        }
    }

    let mut output_str_vec = Vec::new();
    output_str_vec.push("bits 16\n\n".to_owned());
    for ins in outputs {
        output_str_vec.push(ins.output);
    }

    Ok(output_str_vec)
}

#[allow(dead_code)]
fn write_to_file(input_filepath: PathBuf, output: String) -> Result<()> {
    let output_filename = {
        let mut filename = input_filepath
            .file_name()
            .context("Could not locate the filename")?
            .to_str()
            .context("Filename is not valid, could not convert to string")?
            .to_owned();

        filename.push_str("_output.asm");

        filename
    };

    let mut output_filepath = input_filepath.parent().unwrap().to_path_buf();
    output_filepath.push(output_filename);

    let mut output_file = File::create(output_filepath)?;
    output_file.write_all(output.as_bytes())?;

    Ok(())
}

#[allow(dead_code)]
fn write_to_test_file(input_filepath: &str, outputs: Vec<String>) -> Result<String> {
    let input_filepath = PathBuf::from_str(input_filepath)?;
    let output_filename = {
        let mut filename = input_filepath
            .file_name()
            .context("Could not locate the filename")?
            .to_str()
            .context("Filename is not valid, could not convert to string")?
            .to_owned();

        filename.push_str("_test.asm");

        filename
    };

    let mut output_file = File::create(&output_filename)?;
    for output in outputs {
        output_file.write_all(output.as_bytes())?;
    }

    Ok(output_filename)
}

fn main() -> Result<()> {
    let filepath = format!("{}/{}", FILE_DIR, "listing_0041_add_sub_cmp_jnz");

    let mut correct_asm_filepath = filepath.to_owned();
    correct_asm_filepath.push_str(".asm");

    //* assemble correct asm
    execute_nasm(&correct_asm_filepath)?;
    let bytes_of_correct = fs::read(filepath)?;

    let instructions = &bytes_of_correct;

    // println!("{:?}", instructions);

    let outputs = decode_instructions(instructions)?;

    for line in outputs {
        print!("{}", line);
    }

    // write_to_file(input_filepath, output)?;
    // write_to_test_file(output)?;

    Ok(())
}
