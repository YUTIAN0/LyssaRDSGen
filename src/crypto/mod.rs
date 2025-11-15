//! Cryptographic operations module

pub mod curve;
pub mod encoding;
pub mod rc4;

pub use curve::EllipticCurvePoint;
pub use encoding::{decode_pkey, encode_pkey};
pub use rc4::rc4_crypt;

use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Convert BigUint to little-endian bytes with specified length
pub fn bigint_to_bytes_le(n: &BigUint, length: usize) -> Vec<u8> {
    let mut bytes = n.to_bytes_le();
    bytes.resize(length, 0);
    bytes
}

/// Convert little-endian bytes to BigUint
pub fn bytes_to_bigint_le(data: &[u8]) -> BigUint {
    BigUint::from_bytes_le(data)
}

/// Calculate modular multiplicative inverse using Extended Euclidean Algorithm
pub fn mod_inverse(a: &BigUint, m: &BigUint) -> Option<BigUint> {
    use num_bigint::BigInt;
    use num_traits::ToPrimitive;
    
    fn extended_gcd(a: BigInt, b: BigInt) -> (BigInt, BigInt, BigInt) {
        if a.is_zero() {
            return (b, BigInt::from(0), BigInt::from(1));
        }
        let (gcd, x1, y1) = extended_gcd(&b % &a, a.clone());
        let x = y1 - (&b / &a) * &x1;
        let y = x1;
        (gcd, x, y)
    }
    
    let a_int = BigInt::from(a.clone());
    let m_int = BigInt::from(m.clone());
    
    let (gcd, x, _) = extended_gcd(&a_int % &m_int, m_int.clone());
    if gcd != BigInt::from(1) {
        return None;
    }
    
    // Convert result to positive value in range [0, m)
    let result = ((x % &m_int) + &m_int) % &m_int;
    Some(result.to_biguint().unwrap())
}
