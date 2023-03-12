use crate::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;
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

    fn decode_bytes(filepath: &str, should_delete_temp_files: bool) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut correct_asm_filepath = filepath.to_owned();
        correct_asm_filepath.push_str(".asm");
        //* assemble correct asm
        execute_nasm(&correct_asm_filepath).unwrap();

        let correct = fs::read(filepath).unwrap();

        let decoder = Decoder::new();

        let instructions = &correct;
        // println!("{:?}", instructions);

        let mut output = String::new();
        writeln!(output, "bits 16\n").unwrap();

        decode_instructions(&decoder, instructions, &mut output).unwrap();

        let asm_output_filename = write_to_test_file(filepath, output).unwrap();

        execute_nasm(&asm_output_filename).unwrap();

        let output_filename = &asm_output_filename[0..asm_output_filename.len() - 4];
        let bytes_to_test = fs::read(output_filename).unwrap();

        if should_delete_temp_files {
            fs::remove_file(output_filename)?;
            fs::remove_file(asm_output_filename)?;
        }

        Ok((correct, bytes_to_test))
    }

    #[test]
    fn single_register_mov_test() {
        let (correct, bytes_to_test) = decode_bytes(
            "../computer_enhance/perfaware/part1/listing_0037_single_register_mov",
            true,
        )
        .unwrap();

        assert_eq!(correct, bytes_to_test);
    }

    #[test]
    fn many_register_mov_test() {
        let (correct, bytes_to_test) = decode_bytes(
            "../computer_enhance/perfaware/part1/listing_0038_many_register_mov",
            true,
        )
        .unwrap();

        assert_eq!(correct, bytes_to_test);
    }

    #[test]
    fn more_movs_test() {
        let (correct, bytes_to_test) = decode_bytes(
            "../computer_enhance/perfaware/part1/listing_0039_more_movs",
            true,
        )
        .unwrap();

        assert_eq!(correct, bytes_to_test);
    }

    #[test]
    fn challenge_movs_test() {
        let (correct, bytes_to_test) = decode_bytes(
            "../computer_enhance/perfaware/part1/listing_0040_challenge_movs",
            true,
        )
        .unwrap();

        assert_eq!(correct, bytes_to_test);
    }

    #[test]
    fn add_sub_cmp_jnz_test() {
        let (correct, bytes_to_test) = decode_bytes(
            "../computer_enhance/perfaware/part1/listing_0041_add_sub_cmp_jnz",
            true,
        )
        .unwrap();

        assert_eq!(correct, bytes_to_test);
    }
}
