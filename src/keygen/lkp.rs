//! LKP (License Key Pack) generation

use crate::crypto::bigint_to_bytes_le;
use crate::keygen::generate_tskey;
use crate::types::LKPCurve;
use num_bigint::BigUint;

/// Generate LKP (License Key Pack)
pub fn generate_lkp(
    pid: &str,
    count: u32,
    chid: u32,
    major_ver: u32,
    minor_ver: u32,
) -> anyhow::Result<String> {
    if !(1..=9999).contains(&count) {
        anyhow::bail!("License count must be between 1 and 9999");
    }
    
    // Calculate version encoding
    let version = if (major_ver == 5 && minor_ver > 0) || major_ver > 5 {
        (major_ver << 3) | minor_ver
    } else {
        1
    };
    
    // Encode LKP info
    let lkpinfo = ((chid as u64) << 46)
        | ((count as u64) << 32)
        | (2u64 << 18)
        | (144u64 << 10)
        | ((version as u64) << 3);
    
    let lkpdata = bigint_to_bytes_le(&BigUint::from(lkpinfo), 7);
    
    if lkpdata.len() != 7 {
        anyhow::bail!("LKP Info did not convert to 7 bytes");
    }
    
    generate_tskey(
        pid,
        &lkpdata,
        LKPCurve::gx(),
        LKPCurve::gy(),
        BigUint::from(LKPCurve::A),
        LKPCurve::p(),
        LKPCurve::n(),
        LKPCurve::priv_key(),
        1000,
    )
}
