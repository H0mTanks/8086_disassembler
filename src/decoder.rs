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

        //* Set indices to mov as per the machine instruction encoding table
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

    //* Extract instruction fields
    let direction = (first_byte & 0b00000010) > 0;
    let word = (first_byte & 0b00000001) > 0;
    let rm = second_byte & 0b00000111;
    let reg = (second_byte & 0b00111000) >> 3;
    let mode = (second_byte & 0b11000000) >> 6;

    //* Register/Memory to/from register
    {
        //* Test 6 bit opcode
        let opcode = 0b100010;
        if first_byte >> 2 == opcode {
            if mode == 0b11 {
                //* Register to register mode

                let (src, dest) = if direction { (rm, reg) } else { (reg, rm) };

                writeln!(
                    output,
                    "mov {}, {}",
                    get_register_name(dest, word),
                    get_register_name(src, word)
                )?;
            } else {
                //* Memory to register mode, where mode = 00 | 01 | 10
                if mode == 0b00 && rm == 0b110 {
                    //TODO Direct address case
                } else {
                    let mut source_addr_str = String::new();
                    write!(source_addr_str, "{}", MEM_ADDR_MODE_MAPPING[rm as usize])?;

                    //* Check if addr should have displacement
                    //* by checking the mode != 0b00
                    //? Displacements are signed even though the manual says unsigned?
                    if mode != 0b00 {
                        let third_byte = instructions[offset + num_bytes_in_instruction];
                        num_bytes_in_instruction += 1;
                        let mut disp = third_byte as i16;

                        //* Check for 16bit displacement
                        if mode == 0b10 {
                            let fourth_byte = instructions[offset + num_bytes_in_instruction];
                            num_bytes_in_instruction += 1;
                            disp = i16::from_le_bytes([third_byte, fourth_byte]);
                        }

                        //* No need to print displacement if it's 0
                        if disp != 0 {
                            if mode == 0b01 {
                                //* 8 bit displacement
                                write!(source_addr_str, " + {}", disp as i8)?;
                            } else {
                                //* 16 bit displacement
                                write!(source_addr_str, " + {}", disp)?;
                            }
                        }
                    }

                    if direction {
                        writeln!(
                            output,
                            "mov {}, [{}]",
                            get_register_name(reg, word),
                            source_addr_str
                        )?;
                    } else {
                        writeln!(
                            output,
                            "mov [{}], {}",
                            source_addr_str,
                            get_register_name(reg, word),
                        )?;
                    }
                }
            }

            return Ok(num_bytes_in_instruction as u8);
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
                let third_byte = instructions[offset + num_bytes_in_instruction];
                data = u16::from_le_bytes([second_byte, third_byte]);
                num_bytes_in_instruction += 1;
            }

            writeln!(output, "mov {}, {}", get_register_name(reg, word), data)?;

            return Ok(num_bytes_in_instruction as u8);
        }
    }

    //* Immediate to memory
    {
        let opcode = 0b1100011;
        if (first_byte >> 1) == opcode && reg == 0b000 {
            let mut source_addr_str = String::new();
            write!(source_addr_str, "{}", MEM_ADDR_MODE_MAPPING[rm as usize])?;

            //* Check if addr should have displacement
            //* by checking the mode != 0b00
            //? Displacements are signed even though the manual says unsigned?
            if mode != 0b00 {
                let third_byte = instructions[offset + num_bytes_in_instruction];
                num_bytes_in_instruction += 1;
                let mut disp = third_byte as i16;

                //* Check for 16bit displacement
                if mode == 0b10 {
                    let fourth_byte = instructions[offset + num_bytes_in_instruction];
                    num_bytes_in_instruction += 1;
                    disp = i16::from_le_bytes([third_byte, fourth_byte]);
                }

                //* No need to print displacement if it's 0
                if disp != 0 {
                    if mode == 0b01 {
                        //* 8 bit displacement
                        write!(source_addr_str, " + {}", disp as i8)?;
                    } else {
                        //* 16 bit displacement
                        write!(source_addr_str, " + {}", disp)?;
                    }
                }
            }

            let fifth_byte = instructions[offset + num_bytes_in_instruction];
            num_bytes_in_instruction += 1;
            let mut data = fifth_byte as u16;
            //*Check if word or byte operation
            if word {
                let sixth_byte = instructions[offset + num_bytes_in_instruction];
                num_bytes_in_instruction += 1;
                data = u16::from_le_bytes([fifth_byte, sixth_byte]);
                writeln!(output, "mov [{}], word {}", source_addr_str, data)?;
            } else {
                writeln!(output, "mov [{}], byte {}", source_addr_str, data)?;
            }

            return Ok(num_bytes_in_instruction as u8);
        }
    }

    unreachable!()
}
