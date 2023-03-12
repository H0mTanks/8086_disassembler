const REGISTER_NAME_MAPPING: [&str; 16] = [
    "al", "ax", "cl", "cx", "dl", "dx", "bl", "bx", "ah", "sp", "ch", "bp", "dh", "si", "bh", "di",
];

pub fn get_register_name(reg_val: u8, word: bool) -> &'static str {
    let word_val = u8::from(word);

    REGISTER_NAME_MAPPING[(reg_val * 2 + word_val) as usize]
}

pub const MEM_ADDR_MODE_MAPPING: [&str; 8] = [
    "bx + si", "bx + di", "bp + si", "bp + di", "si", "di", "bp", "bx",
];

//* increments num_bytes_in_instruction by one or two depending on boolean flag word */
pub fn get_byte_or_word(
    instructions: &[u8],
    offset: usize,
    num_bytes_in_instruction: &mut usize,
    word: bool,
) -> u16 {
    let first_byte_from_offset = instructions[offset + *num_bytes_in_instruction];
    *num_bytes_in_instruction += 1;

    let value = if word {
        let second_byte_from_offset = instructions[offset + *num_bytes_in_instruction];
        *num_bytes_in_instruction += 1;
        u16::from_le_bytes([first_byte_from_offset, second_byte_from_offset])
    } else {
        first_byte_from_offset as u16
    };

    value
}

pub type NumBytesInInstruction = usize;
