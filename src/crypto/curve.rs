//! Elliptic curve point operations

use num_bigint::BigUint;
use num_traits::Zero;

use super::mod_inverse;

/// Elliptic curve point
#[derive(Clone, Debug)]
pub struct EllipticCurvePoint {
    pub x: BigUint,
    pub y: BigUint,
    pub a: BigUint,
    pub p: BigUint,
    pub infinity: bool,
}

impl EllipticCurvePoint {
    /// Create a new point on the curve
    pub fn new(x: BigUint, y: BigUint, a: BigUint, p: BigUint) -> Self {
        Self {
            x,
            y,
            a,
            p,
            infinity: false,
        }
    }
    
    /// Create point at infinity
    pub fn infinity(a: BigUint, p: BigUint) -> Self {
        Self {
            x: BigUint::zero(),
            y: BigUint::zero(),
            a,
            p,
            infinity: true,
        }
    }
    
    /// Point addition on elliptic curve
    pub fn add(&self, other: &EllipticCurvePoint) -> EllipticCurvePoint {
        if self.infinity {
            return other.clone();
        }
        if other.infinity {
            return self.clone();
        }
        
        let p = &self.p;
        
        let s = if self.x == other.x {
            if self.y == other.y {
                // Point doubling: s = (3*x^2 + a) / (2*y) mod p
                let numerator = (BigUint::from(3u32) * &self.x * &self.x + &self.a) % p;
                let denominator = (BigUint::from(2u32) * &self.y) % p;
                let inv = mod_inverse(&denominator, p).expect("Failed to compute modular inverse");
                (numerator * inv) % p
            } else {
                // Points are inverse of each other
                return EllipticCurvePoint::infinity(self.a.clone(), self.p.clone());
            }
        } else {
            // Point addition: s = (y2 - y1) / (x2 - x1) mod p
            let numerator = if &other.y >= &self.y {
                (&other.y - &self.y) % p
            } else {
                (p + &other.y - &self.y) % p
            };
            let denominator = if &other.x >= &self.x {
                (&other.x - &self.x) % p
            } else {
                (p + &other.x - &self.x) % p
            };
            let inv = mod_inverse(&denominator, p).expect("Failed to compute modular inverse");
            (numerator * inv) % p
        };
        
        // x3 = s^2 - x1 - x2 mod p
        let s_squared = (&s * &s) % p;
        let x_sum = (&self.x + &other.x) % p;
        let x3 = if s_squared >= x_sum {
            (s_squared - x_sum) % p
        } else {
            (p + s_squared - x_sum) % p
        };
        
        // y3 = s * (x1 - x3) - y1 mod p
        let x_diff = if &self.x >= &x3 {
            (&self.x - &x3) % p
        } else {
            (p + &self.x - &x3) % p
        };
        let s_times_diff = (&s * x_diff) % p;
        let y3 = if s_times_diff >= self.y {
            (s_times_diff - &self.y) % p
        } else {
            (p + s_times_diff - &self.y) % p
        };
        
        EllipticCurvePoint::new(x3, y3, self.a.clone(), self.p.clone())
    }
    
    /// Scalar multiplication using double-and-add algorithm
    pub fn mul(&self, scalar: &BigUint) -> EllipticCurvePoint {
        if scalar.is_zero() {
            return EllipticCurvePoint::infinity(self.a.clone(), self.p.clone());
        }
        
        let mut result = EllipticCurvePoint::infinity(self.a.clone(), self.p.clone());
        let mut addend = self.clone();
        let mut k = scalar.clone();
        
        while !k.is_zero() {
            if (&k & BigUint::from(1u32)) == BigUint::from(1u32) {
                result = result.add(&addend);
            }
            addend = addend.add(&addend);
            k >>= 1;
        }
        
        result
    }
}
