//! Key encoding and decoding functions

use crate::types::KCHARS;
use num_bigint::BigUint;
use num_traits::Zero;

/// Encode integer to product key format (base-24 with dashes)
pub fn encode_pkey(n: &BigUint) -> String {
    if n.is_zero() {
        return String::new();
    }
    
    let mut out = String::new();
    let mut num = n.clone();
    let base = BigUint::from(24u32);
    
    while !num.is_zero() {
        let remainder = &num % &base;
        let digits = remainder.to_u32_digits();
        let idx = if digits.is_empty() { 0 } else { digits[0] as usize };
        out.insert(0, KCHARS.chars().nth(idx).unwrap());
        num /= &base;
    }
    
    // Pad to 35 characters
    while out.len() < 35 {
        out.insert(0, KCHARS.chars().next().unwrap());
    }
    
    // Split into groups of 5 with dashes
    let mut result = String::new();
    for (i, ch) in out.chars().enumerate() {
        if i > 0 && i % 5 == 0 {
            result.push('-');
        }
        result.push(ch);
    }
    
    result
}

/// Decode product key format to integer
pub fn decode_pkey(key: &str) -> anyhow::Result<BigUint> {
    let key_string = key.replace('-', "");
    
    if key_string.len() % 5 != 0 {
        anyhow::bail!("Bad key length");
    }
    
    let mut out = BigUint::zero();
    let base = BigUint::from(24u32);
    
    for ch in key_string.chars() {
        let value = KCHARS.find(ch)
            .ok_or_else(|| anyhow::anyhow!("Invalid character: {}", ch))?;
        out = out * &base + BigUint::from(value);
    }
    
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_decode() {
        let num = BigUint::from(12345678901234567890u64);
        let encoded = encode_pkey(&num);
        let decoded = decode_pkey(&encoded).unwrap();
        assert_eq!(num, decoded);
    }
}
