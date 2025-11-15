#!/usr/bin/env python3
"""
LyssaRDSGen - Command Line Tool
Generates Service Provider Keys (SPKs) and License Key Packs (LKPs)
for Microsoft Remote Desktop Services
"""

import argparse
import hashlib
import hmac
import secrets
import sys
from typing import Tuple

# Character set for key encoding (base-24)
KCHARS = "BCDFGHJKMPQRTVWXY2346789"

# License types with descriptions
LICENSE_TYPES = {
    "001_5_0": "Windows 2000 Per Device",
    "002_5_0": "Windows 2000 Internet Connector",
    "003_5_2": "Windows Server 2003 Per User",
    "004_5_2": "Windows Server 2003 Per Device",
    "005_6_0": "Windows Server 2008 (R2) Per Device",
    "006_6_0": "Windows Server 2008 (R2) Per User",
    "009_6_0": "Windows Server 2008 (R2) VDI Standard",
    "010_6_0": "Windows Server 2008 (R2) VDI Premium",
    "016_6_0": "Windows Server 2008 (R2) VDI Suite",
    "011_6_2": "Windows Server 2012 (R2) Per Device",
    "012_6_2": "Windows Server 2012 (R2) Per User",
    "015_6_2": "Windows Server 2012 (R2) VDI Suite",
    "020_10_0": "Windows Server 2016 Per Device",
    "021_10_0": "Windows Server 2016 Per User",
    "022_10_0": "Windows Server 2016 VDI Suite",
    "026_10_1": "Windows Server 2019 Per Device",
    "027_10_1": "Windows Server 2019 Per User",
    "028_10_1": "Windows Server 2019 VDI Suite",
    "029_10_2": "Windows Server 2022 Per Device",
    "030_10_2": "Windows Server 2022 Per User",
    "031_10_2": "Windows Server 2022 VDI Suite",
    "032_10_3": "Windows Server 2025 Per Device",
    "033_10_3": "Windows Server 2025 Per User",
    "034_10_3": "Windows Server 2025 VDI Suite",
}


# Elliptic Curve Parameters for SPK
class SPKCurve:
    a = 1
    b = 0
    p = 21782971228112002125810473336838725345308036616026120243639513697227789232461459408261967852943809534324870610618161
    n = 629063109922370885449
    Gx = 10692194187797070010417373067833672857716423048889432566885309624149667762706899929433420143814127803064297378514651
    Gy = 14587399915883137990539191966406864676102477026583239850923355829082059124877792299572208431243410905713755917185109
    Kx = 3917395608307488535457389605368226854270150445881753750395461980792533894109091921400661704941484971683063487980768
    Ky = 8858262671783403684463979458475735219807686373661776500155868309933327116988404547349319879900761946444470688332645
    priv = 153862071918555979944


# Elliptic Curve Parameters for LKP
class LKPCurve:
    a = 1
    b = 0
    p = 28688293616765795404141427476803815352899912533728694325464374376776313457785622361119232589082131818578591461837297
    n = 675048016158598417213
    Gx = 18999816458520350299014628291870504329073391058325678653840191278128672378485029664052827205905352913351648904170809
    Gy = 7233699725243644729688547165924232430035643592445942846958231777803539836627943189850381859836033366776176689124317
    Kx = 7147768390112741602848314103078506234267895391544114241891627778383312460777957307647946308927283757886117119137500
    Ky = 20525272195909974311677173484301099561025532568381820845650748498800315498040161314197178524020516408371544778243934
    priv = 100266970209474387075


def mod_inverse(a: int, m: int) -> int:
    """Calculate modular multiplicative inverse using Extended Euclidean Algorithm"""
    def extended_gcd(a: int, b: int) -> Tuple[int, int, int]:
        if a == 0:
            return b, 0, 1
        gcd, x1, y1 = extended_gcd(b % a, a)
        x = y1 - (b // a) * x1
        y = x1
        return gcd, x, y
    
    gcd, x, _ = extended_gcd(a % m, m)
    if gcd != 1:
        raise ValueError("Modular inverse does not exist")
    return (x % m + m) % m


class EllipticCurvePoint:
    """Simple elliptic curve point implementation"""
    
    def __init__(self, x: int, y: int, curve):
        self.x = x
        self.y = y
        self.curve = curve
        self.infinity = False
    
    @classmethod
    def infinity_point(cls, curve):
        """Create point at infinity"""
        point = cls(0, 0, curve)
        point.infinity = True
        return point
    
    def __add__(self, other):
        """Point addition on elliptic curve"""
        if self.infinity:
            new_point = EllipticCurvePoint(other.x, other.y, self.curve)
            new_point.infinity = other.infinity
            return new_point
        if other.infinity:
            new_point = EllipticCurvePoint(self.x, self.y, self.curve)
            new_point.infinity = self.infinity
            return new_point
        
        p = self.curve.p
        
        if self.x == other.x:
            if self.y == other.y:
                # Point doubling
                s = (3 * self.x * self.x + self.curve.a) * mod_inverse(2 * self.y, p) % p
            else:
                # Points are inverse of each other
                return EllipticCurvePoint.infinity_point(self.curve)
        else:
            # Point addition
            s = (other.y - self.y) * mod_inverse(other.x - self.x, p) % p
        
        x3 = (s * s - self.x - other.x) % p
        y3 = (s * (self.x - x3) - self.y) % p
        
        return EllipticCurvePoint(x3, y3, self.curve)
    
    def __mul__(self, scalar: int):
        """Scalar multiplication using double-and-add algorithm"""
        if scalar == 0:
            return EllipticCurvePoint.infinity_point(self.curve)
        if scalar < 0:
            raise ValueError("Scalar must be non-negative")
        
        result = EllipticCurvePoint.infinity_point(self.curve)
        addend = self
        
        while scalar:
            if scalar & 1:
                result = result + addend
            addend = addend + addend
            scalar >>= 1
        
        return result


def bigint_to_bytes_le(n: int, length: int) -> bytes:
    """Convert big integer to little-endian bytes"""
    return n.to_bytes(length, byteorder='little')


def bytes_to_bigint_le(data: bytes) -> int:
    """Convert little-endian bytes to big integer"""
    return int.from_bytes(data, byteorder='little')


def encode_pkey(n: int) -> str:
    """Encode integer to product key format"""
    if n < 0:
        raise ValueError("n must be non-negative")
    if n == 0:
        return ""
    
    out = ""
    while n > 0:
        out = KCHARS[n % 24] + out
        n //= 24
    
    out = out.rjust(35, KCHARS[0])
    
    # Split into groups of 5
    segments = [out[i:i+5] for i in range(0, len(out), 5)]
    return "-".join(segments)


def decode_pkey(key: str) -> int:
    """Decode product key format to integer"""
    key_string = key.replace("-", "")
    
    if len(key_string) % 5 != 0:
        raise ValueError("Bad key length")
    
    out = 0
    for char in key_string:
        value = KCHARS.index(char)
        if value == -1:
            raise ValueError(f"Invalid character: {char}")
        out = out * 24 + value
    
    return out


def rc4(key: bytes, data: bytes) -> bytes:
    """RC4 encryption/decryption"""
    s = list(range(256))
    j = 0
    
    # Key scheduling
    for i in range(256):
        j = (j + s[i] + key[i % len(key)]) % 256
        s[i], s[j] = s[j], s[i]
    
    # Pseudo-random generation
    i = j = 0
    result = bytearray()
    for byte in data:
        i = (i + 1) % 256
        j = (j + s[i]) % 256
        s[i], s[j] = s[j], s[i]
        k = s[(s[i] + s[j]) % 256]
        result.append(byte ^ k)
    
    return bytes(result)


def get_spkid(pid: str) -> int:
    """Extract SPK ID from Product ID"""
    if len(pid) < 23:
        raise ValueError("Invalid PID length")
    
    spkid_part1 = pid[10:16]
    spkid_part2 = pid[18:23]
    combined = spkid_part1 + spkid_part2
    spkid_str = combined.split("-")[0]
    
    return int(spkid_str)


def validate_tskey(pid: str, tskey: str, curve, is_spk: bool = True, debug: bool = False) -> bool:
    """Validate a Terminal Services key"""
    try:
        keydata_int = decode_pkey(tskey)
        keydata_bytes = bigint_to_bytes_le(keydata_int, 21)
        
        # Generate RC4 key from PID
        pid_utf16le = pid.encode('utf-16-le')
        md5_digest = hashlib.md5(pid_utf16le).digest()
        rk = md5_digest[:5] + b'\x00' * 11
        
        # Decrypt
        dc_kdata = rc4(rk, keydata_bytes)
        
        if len(dc_kdata) < 21:
            if debug:
                print(f"Debug: dc_kdata length insufficient: {len(dc_kdata)}", file=sys.stderr)
            return False
        
        keydata_inner = dc_kdata[:7]
        sigdata_bytes = dc_kdata[7:]
        sigdata = bytes_to_bigint_le(sigdata_bytes)
        
        h = sigdata & 0x7FFFFFFFFF
        s = (sigdata >> 35) & 0x1FFFFFFFFFFFFFFFFF
        
        # Verify signature
        G = EllipticCurvePoint(curve.Gx, curve.Gy, curve)
        K = EllipticCurvePoint(curve.Kx, curve.Ky, curve)
        
        hK = K * h
        sG = G * s
        R = hK + sG
        
        if R.infinity:
            if debug:
                print("Debug: R is point at infinity", file=sys.stderr)
            return False
        
        Rx_bytes = bigint_to_bytes_le(R.x, 48)
        Ry_bytes = bigint_to_bytes_le(R.y, 48)
        
        sha1_input = keydata_inner + Rx_bytes + Ry_bytes
        md = hashlib.sha1(sha1_input).digest()
        
        part1 = bytes_to_bigint_le(md[:4])
        part2_intermediate = bytes_to_bigint_le(md[4:8])
        part2 = part2_intermediate >> 29
        ht = (part2 << 32) | part1
        
        if debug:
            print(f"Debug: h={h}, ht={ht}, match={h == ht}", file=sys.stderr)
        
        if h != ht:
            return False
        
        if is_spk:
            spkid_from_key = bytes_to_bigint_le(keydata_inner) & 0x1FFFFFFFFFF
            spkid_from_pid = get_spkid(pid)
            if debug:
                print(f"Debug: spkid_from_key={spkid_from_key}, spkid_from_pid={spkid_from_pid}", file=sys.stderr)
            return spkid_from_key == spkid_from_pid
        
        return True
        
    except Exception as e:
        print(f"Validation error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return False


def generate_tskey(pid: str, keydata_inner: bytes, curve, max_attempts: int = 1000) -> str:
    """Generate a Terminal Services key"""
    # Generate RC4 key from PID
    pid_utf16le = pid.encode('utf-16-le')
    md5_digest = hashlib.md5(pid_utf16le).digest()
    rk = md5_digest[:5] + b'\x00' * 11
    
    G = EllipticCurvePoint(curve.Gx, curve.Gy, curve)
    
    for attempt in range(max_attempts):
        # Generate random nonce
        c_nonce = secrets.randbelow(curve.n - 1) + 1
        
        # Calculate R = c_nonce * G
        R = G * c_nonce
        
        # Calculate hash
        Rx_bytes = bigint_to_bytes_le(R.x, 48)
        Ry_bytes = bigint_to_bytes_le(R.y, 48)
        sha1_input = keydata_inner + Rx_bytes + Ry_bytes
        md = hashlib.sha1(sha1_input).digest()
        
        part1 = bytes_to_bigint_le(md[:4])
        part2_intermediate = bytes_to_bigint_le(md[4:8])
        part2 = part2_intermediate >> 29
        h = (part2 << 32) | part1
        
        # Calculate signature
        s = (c_nonce - (curve.priv * h)) % curve.n
        
        # Mask values (69 bits for s, 35 bits for h)
        s_masked = s & 0x1FFFFFFFFFFFFFFFFF
        h_masked = h & 0x7FFFFFFFFF
        
        # Check if s fits in the mask - both conditions must pass
        if s_masked != s or s_masked >= 0x1FFFFFFFFFFFFFFFFF:
            continue
        
        # Encode signature
        sigdata = (s_masked << 35) | h_masked
        sigdata_bytes = bigint_to_bytes_le(sigdata, 14)
        
        pkdata = keydata_inner + sigdata_bytes
        if len(pkdata) != 21:
            continue
        
        # Encrypt
        pke = rc4(rk, pkdata)
        pk = bytes_to_bigint_le(pke[:20])
        pkstr = encode_pkey(pk)
        
        # Validate
        is_spk = (curve.n == SPKCurve.n)
        if validate_tskey(pid, pkstr, curve, is_spk):
            return pkstr
    
    raise RuntimeError(f"Failed to generate valid key after {max_attempts} attempts")


def generate_spk(pid: str) -> str:
    """Generate SPK (License Server ID)"""
    spkid_num = get_spkid(pid)
    spkdata = bigint_to_bytes_le(spkid_num, 7)
    
    if len(spkdata) != 7:
        raise ValueError("SPKID did not convert to 7 bytes")
    
    return generate_tskey(pid, spkdata, SPKCurve)


def generate_lkp(pid: str, count: int, chid: int, major_ver: int, minor_ver: int) -> str:
    """Generate LKP (License Key Pack)"""
    # Calculate version encoding
    version = 1
    if (major_ver == 5 and minor_ver > 0) or major_ver > 5:
        version = (major_ver << 3) | minor_ver
    
    # Encode LKP info
    lkpinfo = (chid << 46) | (count << 32) | (2 << 18) | (144 << 10) | (version << 3)
    lkpdata = bigint_to_bytes_le(lkpinfo, 7)
    
    if len(lkpdata) != 7:
        raise ValueError("LKP Info did not convert to 7 bytes")
    
    return generate_tskey(pid, lkpdata, LKPCurve)


def list_licenses():
    """Print all supported license types"""
    print("\nSupported License Version and Type:\n")
    for code, description in sorted(LICENSE_TYPES.items()):
        print(f"  {code:12s} - {description}")
    print()


def main():
    parser = argparse.ArgumentParser(
        description='LyssaRDSGen - Generate RDS License Keys',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate SPK only
  %(prog)s --pid "00490-92005-99454-AT527"
  
  # Generate both SPK and LKP
  %(prog)s --pid "00490-92005-99454-AT527" --count 1234 --license "029_10_2"
  
  # Use existing SPK to generate LKP
  %(prog)s --pid "00490-92005-99454-AT527" --spk "R2W9K-XKV49-3364Q-92YYM-TKFWJ-DKRCJ-KC2XH" --count 1234 --license "029_10_2"
  
  # List all supported license types
  %(prog)s --list
        """
    )
    
    parser.add_argument('--pid', help='Product ID (e.g., 00490-92005-99454-AT527)')
    parser.add_argument('--spk', help='Existing License Server ID (SPK) - skip SPK generation and only generate LKP')
    parser.add_argument('--count', type=int, help='License count (1-9999) - generates LKP when provided with --license')
    parser.add_argument('--license', help='License version and type (e.g., 029_10_2) - generates LKP when provided with --count')
    parser.add_argument('--list', action='store_true', help='List all supported license types')
    
    args = parser.parse_args()
    
    try:
        # Handle --list flag
        if args.list:
            list_licenses()
            return
        
        # Require PID for key generation
        if not args.pid:
            parser.print_help()
            print("\nError: --pid is required for key generation", file=sys.stderr)
            sys.exit(1)
        
        # Validate --spk parameter requirements
        if args.spk:
            if args.count is None or args.license is None:
                print("Error: When using --spk, both --count and --license must be provided", file=sys.stderr)
                sys.exit(1)
        
        # Validate LKP parameters if either is provided
        if (args.count is None) != (args.license is None):
            print("Error: Both --count and --license must be provided together for LKP generation", file=sys.stderr)
            sys.exit(1)
        
        print(f"Generating keys for PID: {args.pid}\n")
        
        # Handle SPK - either validate existing or generate new
        if args.spk:
            # Validate provided SPK
            print("=" * 60)
            print(f"Validating provided SPK: {args.spk}")
            if not validate_tskey(args.pid, args.spk, SPKCurve, is_spk=True):
                print("=" * 60)
                print("\nError: Provided SPK does not match the PID", file=sys.stderr)
                sys.exit(1)
            spk = args.spk
            print("SPK validation successful!")
            print("=" * 60)
        else:
            # Generate new SPK
            print("=" * 60)
            spk = generate_spk(args.pid)
            print(f"License Server ID (SPK):\n{spk}")
            print("=" * 60)
        
        # Generate LKP if parameters provided
        if args.count is not None and args.license is not None:
            if args.license not in LICENSE_TYPES:
                print(f"\nError: Invalid license type '{args.license}'", file=sys.stderr)
                print("Use --list to see all supported license types", file=sys.stderr)
                sys.exit(1)
            
            if not (1 <= args.count <= 9999):
                print("\nError: License count must be between 1 and 9999", file=sys.stderr)
                sys.exit(1)
            
            # Parse license type
            parts = args.license.split('_')
            if len(parts) != 3:
                print("\nError: License format must be CHID_MAJOR_MINOR (e.g., 029_10_2)", file=sys.stderr)
                sys.exit(1)
            
            chid = int(parts[0])
            major_ver = int(parts[1])
            minor_ver = int(parts[2])
            
            print(f"\nLicense Type: {LICENSE_TYPES[args.license]}")
            print(f"License Count: {args.count}\n")
            print("=" * 60)
            lkp = generate_lkp(args.pid, args.count, chid, major_ver, minor_ver)
            print(f"License Key Pack (LKP):\n{lkp}")
            print("=" * 60)
        
        print()  # Empty line at end
    
    except Exception as e:
        print(f"\nError: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
