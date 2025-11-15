//! Key validation functions

use crate::crypto::{bigint_to_bytes_le, bytes_to_bigint_le, decode_pkey, rc4_crypt, EllipticCurvePoint};
use crate::keygen::get_spkid;
use num_bigint::BigUint;
use sha1::{Digest, Sha1};

/// Validate a Terminal Services key
pub fn validate_tskey(
    pid: &str,
    tskey: &str,
    gx: BigUint,
    gy: BigUint,
    kx: BigUint,
    ky: BigUint,
    a: BigUint,
    p: BigUint,
    is_spk: bool,
) -> anyhow::Result<bool> {
    // Decode key
    let keydata_int = decode_pkey(tskey)?;
    let keydata_bytes = bigint_to_bytes_le(&keydata_int, 21);
    
    // Generate RC4 key from PID
    let pid_utf16le = encode_utf16_le(pid);
    let md5_digest = md5::compute(&pid_utf16le);
    let mut rk = md5_digest[..5].to_vec();
    rk.extend_from_slice(&[0u8; 11]);
    
    // Decrypt
    let dc_kdata = rc4_crypt(&rk, &keydata_bytes);
    
    if dc_kdata.len() < 21 {
        return Ok(false);
    }
    
    let keydata_inner = &dc_kdata[..7];
    let sigdata_bytes = &dc_kdata[7..];
    let sigdata = bytes_to_bigint_le(sigdata_bytes);
    
    let h = &sigdata & BigUint::from(0x7FFFFFFFFFu64);
    let s = (&sigdata >> 35) & BigUint::parse_bytes(b"1FFFFFFFFFFFFFFFFF", 16).unwrap();
    
    // Verify signature
    let g = EllipticCurvePoint::new(gx, gy, a.clone(), p.clone());
    let k = EllipticCurvePoint::new(kx, ky, a, p);
    
    let hk = k.mul(&h);
    let sg = g.mul(&s);
    let r = hk.add(&sg);
    
    if r.infinity {
        return Ok(false);
    }
    
    let rx_bytes = bigint_to_bytes_le(&r.x, 48);
    let ry_bytes = bigint_to_bytes_le(&r.y, 48);
    
    let mut sha1_input = keydata_inner.to_vec();
    sha1_input.extend_from_slice(&rx_bytes);
    sha1_input.extend_from_slice(&ry_bytes);
    
    let md = Sha1::digest(&sha1_input);
    
    let part1 = bytes_to_bigint_le(&md[..4]);
    let part2_intermediate = bytes_to_bigint_le(&md[4..8]);
    let part2 = &part2_intermediate >> 29;
    let ht = (&part2 << 32) | &part1;
    
    if h != ht {
        return Ok(false);
    }
    
    if is_spk {
        let spkid_from_key = bytes_to_bigint_le(keydata_inner) & BigUint::from(0x1FFFFFFFFFFu64);
        let spkid_from_pid = BigUint::from(get_spkid(pid)?);
        return Ok(spkid_from_key == spkid_from_pid);
    }
    
    Ok(true)
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
