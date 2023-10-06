use sha2::{Digest, Sha256};
use std::fmt::Write;

pub fn hash<T: serde::Serialize>(item: &T) -> String {
    let input = serde_json::to_string(&item).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let res = hasher.finalize();
    let vec_res = res.to_vec();

    hex_to_string(vec_res.as_slice())
}

fn hex_to_string(vec_res: &[u8]) -> String {
    let mut s = String::new();
    for b in vec_res {
        write!(&mut s, "{:x}", b).expect("unable to write");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize)]
    struct TestStruct {
        field1: u32,
        field2: String,
    }

    #[test]
    fn test_hash() {
        let test_data = TestStruct {
            field1: 42,
            field2: "hello".to_string(),
        };

        let expected_hash = "dca5c8bcfcd5bc3bf9be36ec693d9b8c27d5e2f51b20d09e75ea4ddad3f24c0";
        let result = hash(&test_data);

        assert_eq!(result, expected_hash);
    }

    #[test]
    fn test_hex_to_string() {
        let byte_array: [u8; 4] = [0x12, 0x34, 0xAB, 0xCD];
        let expected_hex_string = "1234abcd";

        let result = hex_to_string(&byte_array);

        assert_eq!(result, expected_hex_string);
    }

    #[test]
    fn test_hash_with_empty_struct() {
        let empty_struct = TestStruct {
            field1: 0,
            field2: String::new(),
        };

        let expected_hash = "8ee99d7ff199c4a3a1b16b055ecd95d8e8c71b01e78675f6c54c0f81dddc8f0";
        let result = hash(&empty_struct);

        assert_eq!(result, expected_hash);
    }

    // Add more test cases as needed to cover edge cases and different data types.
}