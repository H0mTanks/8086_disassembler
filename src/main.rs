mod constants;
mod decoder;

use std::{
    fmt::Write,
    fs::{self, File},
    io::Write as IoWrite,
    path::PathBuf,
};

pub mod prelude {
    pub use crate::constants::*;
    pub use anyhow::Result;
}

use decoder::*;
use prelude::*;

fn decode_instruction(
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

fn main() -> Result<()> {
    let decoder = Decoder::new();

    let input_filepath =
        PathBuf::from("../computer_enhance/perfaware/part1/listing_0039_more_movs");

    let instructions = fs::read(&input_filepath)?;
    // println!("{:?}", instructions);

    let mut output = String::new();
    writeln!(output, "bits 16\n")?;

    let mut bytes_processed = 0;
    while bytes_processed < instructions.len() {
        bytes_processed +=
            decode_instruction(&decoder, &instructions, bytes_processed, &mut output)? as usize;
    }

    println!("{}", output);

    // write_to_file(input_filepath, output)?;

    Ok(())
}
