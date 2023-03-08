use crate::prelude::*;

use std::fmt::Write;

pub type DecodeFunc =
    fn(instructions: &[u8], offset: usize, output: &mut String) -> Result<NumBytesInInstruction>;

pub struct Decoder {
    pub funcs: [DecodeFunc; 0xF * 0xF],
}

impl Decoder {
    pub fn new() -> Self {
        //*Set all funcs to stub
        let mut funcs = [decode_stub as DecodeFunc; 0xF * 0xF];

        //* Set indices to MOV as per the machine instruction encoding table
        //TODO: 8C, 8E Segment register movs
        funcs[0x88..=0x8B].fill(decode_mov);
        funcs[0xA0..=0xA3].fill(decode_mov);
        funcs[0xB0..=0xBF].fill(decode_mov);
        funcs[0xc6..=0xC7].fill(decode_mov);

        Self { funcs }
    }
}

#[allow(unused_variables, clippy::ptr_arg)]
pub fn decode_stub(
    instructions: &[u8],
    offset: usize,
    output: &mut String,
) -> Result<NumBytesInInstruction> {
    Ok(0)
}

pub fn decode_mov(
    instructions: &[u8],
    offset: usize,
    output: &mut String,
) -> Result<NumBytesInInstruction> {
    let first_byte = instructions[offset];
    let second_byte = instructions[offset + 1];
    let mut num_bytes_in_instruction = 2;

    //*Register/Memory to/from register
    {
        //*Test 6 bit opcode
        let opcode = 0b100010;
        if first_byte >> 2 == opcode {
            let direction = (first_byte & 0b00000010) > 0;
            let word = (first_byte & 0b00000001) > 0;

            let mode = (second_byte & 0b11000000) >> 6;

            if mode == 0b11 {
                //* Register to register mode

                let (src, dest) = if direction {
                    //*rm, reg */
                    (second_byte & 0b00000111, (second_byte & 0b00111000) >> 3)
                } else {
                    ((second_byte & 0b00111000) >> 3, second_byte & 0b00000111)
                };

                writeln!(
                    output,
                    "MOV {}, {}",
                    get_register_name(dest, word),
                    get_register_name(src, word)
                )?;
            }

            return Ok(num_bytes_in_instruction);
        }
    }

    //*Immediate to register
    {
        let opcode = 0b1011;
        if (first_byte >> 4) == opcode {
            let word = (first_byte & 0b00001000) > 0;
            let reg = first_byte & 0b00000111;

            let mut data = second_byte as u16;
            if word {
                let third_byte = instructions[offset + 2];
                data = u16::from_le_bytes([second_byte, third_byte]);
                num_bytes_in_instruction += 1;
            }

            writeln!(output, "MOV {}, {}", get_register_name(reg, word), data)?;

            return Ok(num_bytes_in_instruction);
        }
    }

    unreachable!()
}
