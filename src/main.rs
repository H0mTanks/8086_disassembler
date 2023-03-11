mod constants;
mod decoder;
mod tests;

use execute::Execute;
use std::process::Command;

use std::{
    fmt::Write,
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

fn execute_nasm(asm_filepath: &str) -> Result<()> {
    //*give output to nasm, then open test and compare bytes
    const NASM_PATH: &str = "nasm";

    let mut first_command = Command::new(NASM_PATH);

    first_command.arg(asm_filepath);

    if first_command.execute_check_exit_status_code(0).is_err() {
        bail!(
            "The path `{}` is not a correct FFmpeg executable binary file.",
            NASM_PATH
        );
    }

    Ok(())
}

fn decode_single_instruction(
    decoder: &Decoder,
    instructions: &[u8],
    offset: usize,
    output: &mut String,
) -> Result<NumBytesInInstruction> {
    let first_byte = instructions[offset];
    let num_bytes_in_instruction =
        decoder.funcs[first_byte as usize](instructions, offset, output)?;

    Ok(num_bytes_in_instruction)
}

fn decode_instructions(decoder: &Decoder, instructions: &[u8], output: &mut String) -> Result<()> {
    let mut bytes_processed = 0;
    while bytes_processed < instructions.len() {
        bytes_processed +=
            decode_single_instruction(&decoder, &instructions, bytes_processed, output)? as usize;
    }

    Ok(())
}

#[allow(dead_code)]
fn write_to_file(input_filepath: PathBuf, output: String) -> Result<()> {
    let output_filename = {
        let mut filename = input_filepath
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
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
fn write_to_test_file(input_filepath: &str, output: String) -> Result<String> {
    let input_filepath = PathBuf::from_str(input_filepath)?;
    let output_filename = {
        let mut filename = input_filepath
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        filename.push_str("_test.asm");

        filename
    };

    let mut output_file = File::create(&output_filename)?;
    output_file.write_all(output.as_bytes())?;

    Ok(output_filename)
}
fn main() -> Result<()> {
    let filepath = "../computer_enhance/perfaware/part1/listing_0041_add_sub_cmp_jnz";

    let mut correct_asm_filepath = filepath.to_owned();
    correct_asm_filepath.push_str(".asm");

    //* assemble correct asm
    execute_nasm(&correct_asm_filepath)?;
    let bytes_of_correct = fs::read(filepath)?;

    let decoder = Decoder::new();
    let instructions = &bytes_of_correct;

    // println!("{:?}", instructions);

    let mut output = String::new();
    writeln!(output, "bits 16\n")?;

    decode_instructions(&decoder, instructions, &mut output)?;

    println!("{}", output);

    // write_to_file(input_filepath, output)?;
    // write_to_test_file(output)?;

    Ok(())
}
