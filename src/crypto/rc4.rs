//! RC4 encryption/decryption

/// RC4 encryption/decryption (symmetric)
pub fn rc4_crypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut s: Vec<u8> = (0..=255).collect();
    let mut j: usize = 0;
    
    // Key scheduling algorithm (KSA)
    for i in 0..256 {
        j = (j + s[i] as usize + key[i % key.len()] as usize) % 256;
        s.swap(i, j);
    }
    
    // Pseudo-random generation algorithm (PRGA)
    let mut i: usize = 0;
    j = 0;
    let mut result = Vec::with_capacity(data.len());
    
    for &byte in data {
        i = (i + 1) % 256;
        j = (j + s[i] as usize) % 256;
        s.swap(i, j);
        let k = s[(s[i] as usize + s[j] as usize) % 256];
        result.push(byte ^ k);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rc4_symmetric() {
        let key = b"test_key";
        let plaintext = b"Hello, World!";
        
        let encrypted = rc4_crypt(key, plaintext);
        let decrypted = rc4_crypt(key, &encrypted);
        
        assert_eq!(plaintext, &decrypted[..]);
    }
}
