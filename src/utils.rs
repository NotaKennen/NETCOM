use std::str::FromStr;

/// Converts key bytes into a command string 
/// 
/// Opposite to `string_to_key()`
pub fn key_to_string(key: &[u8]) -> String {
    let mut rettable = String::new();
    for byte in key.iter() {
        rettable.push_str(&byte.to_string());
        rettable.push_str("-");
    }
    let ret = rettable.strip_suffix("-").unwrap();
    return ret.to_string();
}

/// Converts a command string into key bytes
/// 
/// Opposite to `key_to_string()`
pub fn string_to_key(key_str: &str) -> Result<[u8; 32], ()> {
    // Ensure length
    let key_sections: Vec<&str> = key_str.split("-").collect();
    if key_sections.len() != 32 {return Err(())}

    // Construct
    let mut sbox: [u8; 32] = [0; 32];
    for index in 0..32 {
        let intbyte = {
            let u = u8::from_str(key_sections[index]);
            if u.is_err() {return Err(())} else {u.unwrap()}
        };
        sbox[index] = intbyte;
    }

    return Ok(sbox);
}

/// Converts a vector of tags into a command string
pub fn tag_to_string(tags: Vec<String>) -> String {
    let mut rettable = String::new();
    for tag in tags {
        rettable.push_str(&format!("{} ", tag));
    }
    let ret = rettable.trim();
    return ret.to_string();
}

pub fn upgrade_vec(vector: Vec<&str>) -> Vec<String> {
    let mut rettable = vec![];
    for item in vector {
        rettable.push(item.to_string());
    }
    return rettable;
}