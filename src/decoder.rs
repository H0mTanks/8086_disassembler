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
                    //* Direct address case
                    let direct_address =
                        get_byte_or_word(instructions, offset, &mut num_bytes_in_instruction, true)
                            as i16;

                    writeln!(
                        output,
                        "mov {}, [{}]",
                        get_register_name(reg, word),
                        direct_address
                    )?;
                } else {
                    let mut source_addr_str = String::new();
                    write!(source_addr_str, "{}", MEM_ADDR_MODE_MAPPING[rm as usize])?;

                    //* Check if addr should have displacement
                    //* by checking the mode != 0b00
                    //? Displacements are signed even though the manual says unsigned?
                    if mode != 0b00 {
                        //* Check for 16bit displacement
                        let signed_disp = if mode == 0b10 {
                            get_byte_or_word(
                                instructions,
                                offset,
                                &mut num_bytes_in_instruction,
                                true,
                            ) as i16
                        } else {
                            get_byte_or_word(
                                instructions,
                                offset,
                                &mut num_bytes_in_instruction,
                                false,
                            ) as i16
                        };

                        //* No need to print displacement if it's 0
                        if signed_disp != 0 {
                            if mode == 0b01 {
                                //* 8 bit displacement
                                write!(source_addr_str, " + {}", signed_disp as i8)?;
                            } else {
                                //* 16 bit displacement
                                write!(source_addr_str, " + {}", signed_disp)?;
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

            let data = if word {
                let third_byte = instructions[offset + num_bytes_in_instruction];
                num_bytes_in_instruction += 1;
                u16::from_le_bytes([second_byte, third_byte])
            } else {
                second_byte as u16
            };

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
                let signed_disp = if mode == 0b10 {
                    get_byte_or_word(instructions, offset, &mut num_bytes_in_instruction, true)
                        as i16
                } else {
                    get_byte_or_word(instructions, offset, &mut num_bytes_in_instruction, false)
                        as i16
                };

                //* No need to print displacement if it's 0
                if signed_disp != 0 {
                    if mode == 0b01 {
                        //* 8 bit displacement
                        write!(source_addr_str, " + {}", signed_disp as i8)?;
                    } else {
                        //* 16 bit displacement
                        write!(source_addr_str, " + {}", signed_disp)?;
                    }
                }
            }

            let data = get_byte_or_word(instructions, offset, &mut num_bytes_in_instruction, word);

            if word {
                writeln!(output, "mov [{}], word {}", source_addr_str, data)?;
            } else {
                writeln!(output, "mov [{}], byte {}", source_addr_str, data)?;
            }

            return Ok(num_bytes_in_instruction as u8);
        }
    }

    //* Memory to accumulator or accumulator to memory
    {
        let opcode = 0b101000;
        if (first_byte >> 2) == opcode {
            let is_acc_to_mem = direction;

            let third_byte = instructions[offset + num_bytes_in_instruction];
            num_bytes_in_instruction += 1;
            let direct_address = i16::from_le_bytes([second_byte, third_byte]);

            if is_acc_to_mem {
                writeln!(
                    output,
                    "mov [{}], {}",
                    direct_address,
                    get_register_name(0, word)
                )?;
            } else {
                //* memory to accumulator case
                writeln!(
                    output,
                    "mov {}, [{}]",
                    get_register_name(0, word),
                    direct_address,
                )?;
            }

            return Ok(num_bytes_in_instruction as u8);
        }
    }

    unreachable!()
}
