pub mod strings {
    pub fn bytes_to_hex(buffer: &[u8]) -> String {
        if buffer.is_empty() {
            return "0x".to_string();
        }

        let converted: Vec<u8> = buffer
            .iter()
            .flat_map(|byte| vec![byte / 16, byte % 16])
            .collect();

        let base: Vec<u8> = b"0123456789abcdef".to_vec();

        let hex_string: String = converted
            .iter()
            .map(|byte| base[*byte as usize] as char)
            .collect();

        format!("0x{}", hex_string)
    }

    pub fn concat(base: &str, value: &str) -> String {
        format!("{}{}", base, value)
    }

    pub fn index_of(base: &str, value: &str) -> Option<usize> {
        base.find(value)
    }

    pub fn length(base: &str) -> usize {
        base.len()
    }

    pub fn split(base: &str, value: &str) -> Vec<String> {
        base.split(value).map(|s| s.to_string()).collect()
    }

    pub fn compare_to(base: &str, value: &str) -> bool {
        base == value
    }

    pub fn lower(base: &str) -> String {
        base.to_lowercase()
    }
}
