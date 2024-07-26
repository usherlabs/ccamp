pub mod ecdsa;
pub mod ethereum;
pub mod redstone;
pub mod streamr;

pub fn string_to_vec_u8(str: &str) -> Vec<u8> {
    let starts_from: usize;
    if str.starts_with("0x") {
        starts_from = 2;
    } else {
        starts_from = 0;
    }

    (starts_from..str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&str[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>()
}

pub fn vec_u8_to_string(vec: &Vec<u8>) -> String {
    vec.iter()
        .map(|r| format!("{:02x}", r))
        .collect::<Vec<String>>()
        .join("")
        .to_string()
}

pub fn remove_leading(vec: &Vec<u8>, element: u8) -> Vec<u8> {
    let start = vec.iter().position(|&x| x != element).unwrap();
    let result = &vec[start..];
    result.to_vec()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_vec_u8(){
        let hex_string = "0x5c8e3a7c16fa5cdde9f74751d6b2395176f05c55";
        let hex_output = [92, 142, 58, 124, 22, 250, 92, 221, 233, 247, 71, 81, 214, 178, 57, 81, 118, 240, 92, 85].to_vec();


        let output_vector = string_to_vec_u8(&hex_string);
        assert_eq!(hex_output, output_vector);
    }

    #[test]
    fn test_vec_u8_to_string(){
        let hex_string = "5c8e3a7c16fa5cdde9f74751d6b2395176f05c55";
        let hex_output = [92, 142, 58, 124, 22, 250, 92, 221, 233, 247, 71, 81, 214, 178, 57, 81, 118, 240, 92, 85].to_vec();


        let output_hex_string = vec_u8_to_string(&hex_output);
        assert_eq!(hex_string, output_hex_string);
    }

}