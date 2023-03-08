use std::{
    fmt::{Display, Write},
    fs::{self, File},
    io::Write as IoWrite,
    path::{Path, PathBuf},
};

use bitvec::prelude::*;

use arbitrary_int::{u2, u3, u6};
use bitbybit::bitfield;

use anyhow::{bail, Result};
enum Opcode {
    MOV,
}

const REGISTER_NAME_MAPPING: [&str; 16] = [
    "AL", "AX", "CL", "CX", "DL", "DX", "BL", "BX", "AH", "SP", "CH", "BP", "DH", "SI", "BH", "DI",
];

fn get_register_name(reg_val: u3, word: bool) -> &'static str {
    let word_val = u8::from(word);
    let reg_val = u8::from(reg_val);

    REGISTER_NAME_MAPPING[(reg_val * 2 + word_val) as usize]
}

fn decode_mov(instructions: &Vec<u8>, offset: usize, output: &mut String) -> Result<u8> {
    let first_byte = FirstByte::new_with_raw_value(instructions[offset]);
    let second_byte = SecondByte::new_with_raw_value(instructions[offset + 1]);
    let mut num_bytes_in_instruction = 2;

    let (src, dest) = if first_byte.direction() {
        (second_byte.rm(), second_byte.reg())
    } else {
        (second_byte.reg(), second_byte.rm())
    };

    writeln!(
        output,
        "MOV {}, {}",
        get_register_name(dest, first_byte.word()),
        get_register_name(src, first_byte.word())
    )?;

    Ok(num_bytes_in_instruction)
}

fn decode_instruction(instructions: &Vec<u8>, offset: usize, output: &mut String) -> Result<u8> {
    // let first_byte = FirstByte::new_with_raw_value(instructions[offset]);
    let first_byte = instructions[offset];
    let first_byte_bits = first_byte.view_bits::<Msb0>();
    let num_bytes_in_instruction = 1;

    let mut bit_idx = 0;
    if first_byte_bits[bit_idx] {
        //* 1
        bit_idx += 1;
    } else {
        //* 0
        bit_idx += 1;
    }

    // let opcode = {
    //     let opcode_val = first_byte.opcode().value();
    //     match opcode_val {
    //         0b100010 | 0b1100011 | 0b1011 | 0b101000 | 0b100011 => Opcode::MOV,
    //         _ => {
    //             bail!("Invalid opcode: {}", first_byte.opcode())
    //         }
    //     }
    // };

    // let num_bytes_in_instruction = match opcode {
    //     Opcode::MOV => decode_mov(instructions, offset, output)?,
    // };

    Ok(num_bytes_in_instruction)
}

#[bitfield(u8, default: 0)]
struct FirstByte {
    #[bits(2..=7, rw)]
    opcode: u6,

    #[bit(1, rw)]
    direction: bool,

    #[bit(0, rw)]
    word: bool,
}

// #[bitfield(u8, default: 0)]
// struct FirstByte {
//     #[bits(2..=7, rw)]
//     opcode: u6,

//     #[bit(1, rw)]
//     direction: bool,

//     #[bit(0, rw)]
//     word: bool,
// }

#[bitfield(u8, default: 0)]
struct SecondByte {
    #[bits(6..=7, rw)]
    mode: u2,

    #[bits(3..=5, rw)]
    reg: u3,

    #[bits(0..=2, rw)]
    rm: u3,
}

impl Display for FirstByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n  {}: {:b},\n  {}: {},\n  {}: {},\n}}",
            "opcode",
            self.opcode().value(),
            "direction",
            if self.direction() { 1 } else { 0 },
            "word",
            if self.word() { 1 } else { 0 },
        )
    }
}

impl Display for SecondByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n  {}: {:b},\n  {}: {},\n  {}: {},\n}}",
            "mode",
            self.mode().value(),
            "reg",
            self.reg().value(),
            "rm",
            self.rm().value(),
        )
    }
}

fn main() -> Result<()> {
    let input_filepath =
        PathBuf::from("../computer_enhance/perfaware/part1/listing_0038_many_register_mov");

    let instructions = fs::read(&input_filepath)?;
    println!("{:?}", instructions);

    let mut output = String::new();
    writeln!(output, "bits 16\n")?;

    let mut bytes_processed = 0;
    while bytes_processed < instructions.len() {
        bytes_processed +=
            decode_instruction(&instructions, bytes_processed, &mut output)? as usize;
    }

    println!("{}", output);

    // let output_filename = {
    //     let mut filename = input_filepath
    //         .file_name()
    //         .unwrap()
    //         .to_str()
    //         .unwrap()
    //         .to_owned();

    //     filename.push_str("_output.asm");

    //     filename
    // };

    // let mut output_filepath = input_filepath.parent().unwrap().to_path_buf();
    // output_filepath.push(output_filename);

    // let mut output_file = File::create(output_filepath)?;
    // output_file.write_all(output.as_bytes())?;

    let data = 0xAAu8;
    let bits = data.view_bits::<Msb0>();

    for bit in bits {
        println!("{}", bit);
    }

    Ok(())
}
