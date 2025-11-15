//! Key generation module

pub mod lkp;
pub mod spk;
pub mod validation;

pub use lkp::generate_lkp;
pub use spk::generate_spk;
pub use validation::validate_tskey;

use crate::crypto::{bigint_to_bytes_le, bytes_to_bigint_le, encode_pkey, rc4_crypt, EllipticCurvePoint};
use num_bigint::BigUint;
use num_traits::Zero;
use rand::Rng;
use sha1::{Digest, Sha1};

/// Extract SPK ID from Product ID
pub fn get_spkid(pid: &str) -> anyhow::Result<u64> {
    if pid.len() < 23 {
        anyhow::bail!("Invalid PID length");
    }
    
    let spkid_part1 = &pid[10..16];
    let spkid_part2 = &pid[18..23];
    let combined = format!("{}{}", spkid_part1, spkid_part2);
    let spkid_str = combined.split('-').next().unwrap_or("");
    
    spkid_str.parse::<u64>()
        .map_err(|e| anyhow::anyhow!("Failed to parse SPKID: {}", e))
}

/// Generate Terminal Services key (generic function for both SPK and LKP)
pub fn generate_tskey(
    pid: &str,
    keydata_inner: &[u8],
    gx: BigUint,
    gy: BigUint,
    a: BigUint,
    p: BigUint,
    n: BigUint,
    priv_key: BigUint,
    max_attempts: usize,
) -> anyhow::Result<String> {
    // Determine if this is SPK based on curve parameters
    let is_spk = n == crate::types::SPKCurve::n();
    // Generate RC4 key from PID
    let pid_utf16le = encode_utf16_le(pid);
    let md5_digest = md5::compute(&pid_utf16le);
    let mut rk = md5_digest[..5].to_vec();
    rk.extend_from_slice(&[0u8; 11]);
    
    let g = EllipticCurvePoint::new(gx.clone(), gy.clone(), a.clone(), p.clone());
    
    for _ in 0..max_attempts {
        // Generate random nonce
        let mut rng = rand::thread_rng();
        let c_nonce = BigUint::from(rng.gen::<u64>() % n.to_u64_digits()[0]) + BigUint::from(1u32);
        
        // Calculate R = c_nonce * G
        let r = g.mul(&c_nonce);
        
        // Calculate hash
        let rx_bytes = bigint_to_bytes_le(&r.x, 48);
        let ry_bytes = bigint_to_bytes_le(&r.y, 48);
        
        let mut sha1_input = keydata_inner.to_vec();
        sha1_input.extend_from_slice(&rx_bytes);
        sha1_input.extend_from_slice(&ry_bytes);
        
        let md = Sha1::digest(&sha1_input);
        
        let part1 = bytes_to_bigint_le(&md[..4]);
        let part2_intermediate = bytes_to_bigint_le(&md[4..8]);
        let part2 = &part2_intermediate >> 29;
        let h = (&part2 << 32) | &part1;
        
        // Calculate signature: s = (c_nonce - priv_key * h) mod n
        let s = if &c_nonce >= &(&priv_key * &h % &n) {
            (&c_nonce - (&priv_key * &h % &n)) % &n
        } else {
            (&n + &c_nonce - (&priv_key * &h % &n)) % &n
        };
        
        // Mask values (69 bits for s, 35 bits for h)
        let s_mask = BigUint::parse_bytes(b"1FFFFFFFFFFFFFFFFF", 16).unwrap();
        let h_mask = BigUint::from(0x7FFFFFFFFFu64);
        
        let s_masked = &s & &s_mask;
        let h_masked = &h & &h_mask;
        
        // Check if s fits in the mask
        if s_masked != s || s_masked >= s_mask {
            continue;
        }
        
        // Encode signature
        let sigdata = (&s_masked << 35) | &h_masked;
        let sigdata_bytes = bigint_to_bytes_le(&sigdata, 14);
        
        let mut pkdata = keydata_inner.to_vec();
        pkdata.extend_from_slice(&sigdata_bytes);
        
        if pkdata.len() != 21 {
            continue;
        }
        
        // Encrypt
        let pke = rc4_crypt(&rk, &pkdata);
        let pk = bytes_to_bigint_le(&pke[..20]);
        let pkstr = encode_pkey(&pk);
        
        // Validate the generated key
        match validate_tskey(
            pid,
            &pkstr,
            gx.clone(),
            gy.clone(),
            // For validation, we need Kx and Ky (public key)
            if is_spk {
                crate::types::SPKCurve::kx()
            } else {
                crate::types::LKPCurve::kx()
            },
            if is_spk {
                crate::types::SPKCurve::ky()
            } else {
                crate::types::LKPCurve::ky()
            },
            a.clone(),
            p.clone(),
            is_spk,
        ) {
            Ok(true) => return Ok(pkstr),
            _ => continue,
        }
    }
    
    anyhow::bail!("Failed to generate valid key after {} attempts", max_attempts)
}

/// Encode string to UTF-16 LE bytes
fn encode_utf16_le(s: &str) -> Vec<u8> {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    let mut bytes = Vec::with_capacity(utf16.len() * 2);
    for word in utf16 {
        bytes.push((word & 0xFF) as u8);
        bytes.push((word >> 8) as u8);
    }
    bytes
}
