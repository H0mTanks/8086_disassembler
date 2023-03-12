use anyhow::Ok;

use crate::prelude::*;

use std::fmt::Write;

pub type DecodeFunc = fn(
    instructions: &[u8],
    offset: usize,
    output: &mut String,
    decoder: &Decoder,
) -> Result<NumBytesInInstruction>;

pub struct Decoder {
    pub funcs: [DecodeFunc; 0xF * 0xF],
    pub groups: [DecodeFunc; 8 * 4],
}

impl Decoder {
    pub fn new() -> Self {
        let mut groups = [decode_stub as DecodeFunc; 8 * 4];

        //*Set all funcs to stub
        let mut funcs = [decode_stub as DecodeFunc; 0xF * 0xF];

        //* Set indices to MOV as per the machine instruction encoding table
        //TODO: 8C, 8E Segment register movs
        funcs[0x88..=0x8B].fill(decode_mov);
        funcs[0xA0..=0xA3].fill(decode_mov);
        funcs[0xB0..=0xBF].fill(decode_mov);
        funcs[0xc6..=0xC7].fill(decode_mov);

        //* ADD indices
        funcs[0x0..=0x5].fill(decode_add);
        funcs[0x80..=0x83].fill(decode_from_group);
        groups[0b000] = decode_add;

        Self { funcs, groups }
    }
}

pub fn decode_from_group(
    instructions: &[u8],
    offset: usize,
    output: &mut String,
    decoder: &Decoder,
) -> Result<NumBytesInInstruction> {
    let first_byte = instructions[offset];

    let row = if first_byte >= 0x80 && first_byte <= 0x83 {
        0
    } else if first_byte >= 0xD0 && first_byte <= 0xD3 {
        1
    } else if first_byte == 0xF6 || first_byte == 0xF7 {
        2
    } else if first_byte == 0xFE || first_byte == 0xFF {
        3
    } else {
        bail!("Invalid first_byte opcode");
    };

    let second_byte = instructions[offset + 1];
    let reg = (second_byte & 0b00111000) >> 3;

    // println!("{}", row * 8 + reg);
    let num_bytes_in_instruction =
        decoder.groups[(row * 8 + reg) as usize](instructions, offset, output, decoder)?;

    Ok(num_bytes_in_instruction)
}

#[allow(unused_variables, clippy::ptr_arg)]
pub fn decode_stub(
    instructions: &[u8],
    offset: usize,
    output: &mut String,
    _: &Decoder,
) -> Result<NumBytesInInstruction> {
    Ok(0)
}

pub fn decode_mov(
    instructions: &[u8],
    offset: usize,
    output: &mut String,
    _: &Decoder,
) -> Result<NumBytesInInstruction> {
    let first_byte = instructions[offset];
    let mut num_bytes_in_instruction = 1;

    //* Register/Memory to/from register
    {
        //* Test 6 bit opcode
        let opcode = 0b100010;
        if first_byte >> 2 == opcode {
            num_bytes_in_instruction = reg_mem_to_reg_mem("mov", instructions, offset, output)?;
            return Ok(num_bytes_in_instruction);
        }
    }

    //*Immediate to register
    {
        let opcode = 0b1011;
        if (first_byte >> 4) == opcode {
            let word = (first_byte & 0b00001000) > 0;
            let reg = first_byte & 0b00000111;

            let data = get_byte_or_word(instructions, offset, &mut num_bytes_in_instruction, word);

            writeln!(output, "mov {}, {}", get_register_name(reg, word), data)?;

            return Ok(num_bytes_in_instruction);
        }
    }

    //* Immediate to memory
    {
        let second_byte = instructions[offset + num_bytes_in_instruction as usize];
        let reg = (second_byte & 0b00111000) >> 3;

        let opcode = 0b1100011;
        if (first_byte >> 1) == opcode && reg == 0b000 {
            num_bytes_in_instruction += 1;

            //*Extract fields
            let word = (first_byte & 0b00000001) > 0;
            let rm = second_byte & 0b00000111;
            let mode = (second_byte & 0b11000000) >> 6;

            imm_to_mem(
                "mov",
                instructions,
                offset,
                &mut num_bytes_in_instruction,
                mode,
                rm,
                word,
                word,
                output,
            )?;

            return Ok(num_bytes_in_instruction);
        }
    }

    //* Memory to accumulator or accumulator to memory
    {
        let opcode = 0b101000;
        if (first_byte >> 2) == opcode {
            let direction = (first_byte & 0b00000010) > 0;
            let word = (first_byte & 0b00000001) > 0;
            let is_acc_to_mem = direction;

            let direct_address =
                get_byte_or_word(instructions, offset, &mut num_bytes_in_instruction, word) as i16;

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

            return Ok(num_bytes_in_instruction);
        }
    }

    unreachable!()
}

pub fn decode_add(
    instructions: &[u8],
    offset: usize,
    output: &mut String,
    _: &Decoder,
) -> Result<NumBytesInInstruction> {
    let first_byte = instructions[offset];
    let mut num_bytes_in_instruction = 1;

    //* Reg/Memory with register to either
    {
        let opcode = 0b000000;
        if (first_byte >> 2) == opcode {
            num_bytes_in_instruction = reg_mem_to_reg_mem("add", instructions, offset, output)?;
            return Ok(num_bytes_in_instruction);
        }
    }

    //* Immediate to register/memory
    {
        let second_byte = instructions[offset + num_bytes_in_instruction as usize];

        let reg = (second_byte & 0b00111000) >> 3;
        let opcode = 0b100000;
        if (first_byte >> 2) == opcode && reg == 0b000 {
            num_bytes_in_instruction += 1;

            //*Extract fields
            let sign = (first_byte & 0b00000010) > 0;
            let word = (first_byte & 0b00000001) > 0;
            let rm = second_byte & 0b00000111;
            let mode = (second_byte & 0b11000000) >> 6;

            if mode == 0b11 {
                //* Immediate to register case
                let is_data_16bit = !sign && word;

                let data = get_byte_or_word(
                    instructions,
                    offset,
                    &mut num_bytes_in_instruction,
                    is_data_16bit,
                );

                writeln!(output, "add {}, {}", get_register_name(rm, word), data)?;
            } else {
                let is_data_16bit = !sign && word;

                //* Immediate to memory case
                imm_to_mem(
                    "add",
                    instructions,
                    offset,
                    &mut num_bytes_in_instruction,
                    mode,
                    rm,
                    word,
                    is_data_16bit,
                    output,
                )?;
            }
            return Ok(num_bytes_in_instruction);
        }
    }

    //* Immediate to accumulator
    {
        let opcode = 0b0000010;
        if (first_byte >> 1) == opcode {
            let word = (first_byte & 0b00000001) > 0;

            let data = get_byte_or_word(instructions, offset, &mut num_bytes_in_instruction, word);

            writeln!(output, "add {}, {}", get_register_name(0, word), data)?;

            return Ok(num_bytes_in_instruction);
        }
    }
    unreachable!()
}

fn register_to_register(
    op_name: &str,
    reg: u8,
    rm: u8,
    direction: bool,
    word: bool,
    output: &mut String,
) -> Result<()> {
    let (src, dest) = if direction { (rm, reg) } else { (reg, rm) };

    writeln!(
        output,
        "{} {}, {}",
        op_name,
        get_register_name(dest, word),
        get_register_name(src, word)
    )?;

    Ok(())
}

fn reg_mem_to_reg_mem(
    opname: &str,
    instructions: &[u8],
    offset: usize,
    output: &mut String,
) -> Result<NumBytesInInstruction> {
    let first_byte = instructions[offset];
    let mut num_bytes_in_instruction = 1;

    let second_byte = instructions[offset + num_bytes_in_instruction];
    num_bytes_in_instruction += 1;

    //* Extract instruction fields
    let direction = (first_byte & 0b00000010) > 0;
    let word = (first_byte & 0b00000001) > 0;
    let rm = second_byte & 0b00000111;
    let reg = (second_byte & 0b00111000) >> 3;
    let mode = (second_byte & 0b11000000) >> 6;

    if mode == 0b11 {
        //* Register to register mode
        register_to_register(opname, reg, rm, direction, word, output)?;
    } else {
        //* Memory to register mode, where mode = 00 | 01 | 10
        if mode == 0b00 && rm == 0b110 {
            //* Direct address case
            let direct_address =
                get_byte_or_word(instructions, offset, &mut num_bytes_in_instruction, true) as i16;

            writeln!(
                output,
                "{} {}, [{}]",
                opname,
                get_register_name(reg, word),
                direct_address
            )?;
        } else {
            let source_addr_str = construct_address(
                instructions,
                offset,
                &mut num_bytes_in_instruction,
                mode,
                rm,
            )?;

            if direction {
                writeln!(
                    output,
                    "{} {}, [{}]",
                    opname,
                    get_register_name(reg, word),
                    source_addr_str
                )?;
            } else {
                writeln!(
                    output,
                    "{} [{}], {}",
                    opname,
                    source_addr_str,
                    get_register_name(reg, word),
                )?;
            }
        }
    }

    Ok(num_bytes_in_instruction)
}

fn construct_address(
    instructions: &[u8],
    offset: usize,
    num_bytes_in_instruction: &mut usize,
    mode: u8,
    rm: u8,
) -> Result<String> {
    let mut source_addr_str = String::new();
    write!(source_addr_str, "{}", MEM_ADDR_MODE_MAPPING[rm as usize])?;

    //* Check if addr should have displacement
    //* by checking the mode != 0b00
    //? Displacements are signed even though the manual says unsigned?
    if mode != 0b00 {
        let signed_disp = if mode == 0b10 {
            get_byte_or_word(instructions, offset, num_bytes_in_instruction, true) as i16
        } else {
            get_byte_or_word(instructions, offset, num_bytes_in_instruction, false) as i16
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

    Ok(source_addr_str)
}

fn imm_to_mem(
    opname: &str,
    instructions: &[u8],
    offset: usize,
    num_bytes_in_instruction: &mut usize,
    mode: u8,
    rm: u8,
    word: bool,
    is_data_16bit: bool,
    output: &mut String,
) -> Result<()> {
    let source_addr_str =
        construct_address(instructions, offset, num_bytes_in_instruction, mode, rm)?;

    let data = get_byte_or_word(
        instructions,
        offset,
        num_bytes_in_instruction,
        is_data_16bit,
    );

    if word {
        writeln!(output, "{} [{}], word {}", opname, source_addr_str, data)?;
    } else {
        writeln!(output, "{} [{}], byte {}", opname, source_addr_str, data)?;
    }

    Ok(())
}
