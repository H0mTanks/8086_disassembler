const REGISTER_NAME_MAPPING: [&str; 16] = [
    "AL", "AX", "CL", "CX", "DL", "DX", "BL", "BX", "AH", "SP", "CH", "BP", "DH", "SI", "BH", "DI",
];

pub fn get_register_name(reg_val: u8, word: bool) -> &'static str {
    let word_val = u8::from(word);

    REGISTER_NAME_MAPPING[(reg_val * 2 + word_val) as usize]
}

pub const MEM_ADDR_MODE_MAPPING: [&str; 8] = [
    "bx + si", "bx + di", "bp + si", "bp + di", "si", "di", "bp", "bx",
];

pub type NumBytesInInstruction = u8;
