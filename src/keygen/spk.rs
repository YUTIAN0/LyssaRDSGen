//! SPK (Service Provider Key) generation

use crate::crypto::bigint_to_bytes_le;
use crate::keygen::{generate_tskey, get_spkid};
use crate::types::SPKCurve;
use num_bigint::BigUint;

/// Generate SPK (License Server ID)
pub fn generate_spk(pid: &str) -> anyhow::Result<String> {
    let spkid_num = get_spkid(pid)?;
    let spkdata = bigint_to_bytes_le(&BigUint::from(spkid_num), 7);
    
    if spkdata.len() != 7 {
        anyhow::bail!("SPKID did not convert to 7 bytes");
    }
    
    generate_tskey(
        pid,
        &spkdata,
        SPKCurve::gx(),
        SPKCurve::gy(),
        BigUint::from(SPKCurve::A),
        SPKCurve::p(),
        SPKCurve::n(),
        SPKCurve::priv_key(),
        1000,
    )
}
