//! Common types and constants

use num_bigint::BigUint;

/// Character set for key encoding (base-24)
pub const KCHARS: &str = "BCDFGHJKMPQRTVWXY2346789";

/// License types with descriptions
pub const LICENSE_TYPES: &[(&str, &str)] = &[
    ("001_5_0", "Windows 2000 Per Device"),
    ("002_5_0", "Windows 2000 Internet Connector"),
    ("003_5_2", "Windows Server 2003 Per User"),
    ("004_5_2", "Windows Server 2003 Per Device"),
    ("005_6_0", "Windows Server 2008 (R2) Per Device"),
    ("006_6_0", "Windows Server 2008 (R2) Per User"),
    ("009_6_0", "Windows Server 2008 (R2) VDI Standard"),
    ("010_6_0", "Windows Server 2008 (R2) VDI Premium"),
    ("016_6_0", "Windows Server 2008 (R2) VDI Suite"),
    ("011_6_2", "Windows Server 2012 (R2) Per Device"),
    ("012_6_2", "Windows Server 2012 (R2) Per User"),
    ("015_6_2", "Windows Server 2012 (R2) VDI Suite"),
    ("020_10_0", "Windows Server 2016 Per Device"),
    ("021_10_0", "Windows Server 2016 Per User"),
    ("022_10_0", "Windows Server 2016 VDI Suite"),
    ("026_10_1", "Windows Server 2019 Per Device"),
    ("027_10_1", "Windows Server 2019 Per User"),
    ("028_10_1", "Windows Server 2019 VDI Suite"),
    ("029_10_2", "Windows Server 2022 Per Device"),
    ("030_10_2", "Windows Server 2022 Per User"),
    ("031_10_2", "Windows Server 2022 VDI Suite"),
    ("032_10_3", "Windows Server 2025 Per Device"),
    ("033_10_3", "Windows Server 2025 Per User"),
    ("034_10_3", "Windows Server 2025 VDI Suite"),
];

/// Elliptic curve parameters for SPK
#[derive(Clone)]
pub struct SPKCurve;

impl SPKCurve {
    pub const A: u32 = 1;
    pub const B: u32 = 0;
    
    pub fn p() -> BigUint {
        BigUint::parse_bytes(
            b"21782971228112002125810473336838725345308036616026120243639513697227789232461459408261967852943809534324870610618161",
            10
        ).unwrap()
    }
    
    pub fn n() -> BigUint {
        BigUint::parse_bytes(b"629063109922370885449", 10).unwrap()
    }
    
    pub fn gx() -> BigUint {
        BigUint::parse_bytes(
            b"10692194187797070010417373067833672857716423048889432566885309624149667762706899929433420143814127803064297378514651",
            10
        ).unwrap()
    }
    
    pub fn gy() -> BigUint {
        BigUint::parse_bytes(
            b"14587399915883137990539191966406864676102477026583239850923355829082059124877792299572208431243410905713755917185109",
            10
        ).unwrap()
    }
    
    pub fn kx() -> BigUint {
        BigUint::parse_bytes(
            b"3917395608307488535457389605368226854270150445881753750395461980792533894109091921400661704941484971683063487980768",
            10
        ).unwrap()
    }
    
    pub fn ky() -> BigUint {
        BigUint::parse_bytes(
            b"8858262671783403684463979458475735219807686373661776500155868309933327116988404547349319879900761946444470688332645",
            10
        ).unwrap()
    }
    
    pub fn priv_key() -> BigUint {
        BigUint::parse_bytes(b"153862071918555979944", 10).unwrap()
    }
}

/// Elliptic curve parameters for LKP
#[derive(Clone)]
pub struct LKPCurve;

impl LKPCurve {
    pub const A: u32 = 1;
    pub const B: u32 = 0;
    
    pub fn p() -> BigUint {
        BigUint::parse_bytes(
            b"28688293616765795404141427476803815352899912533728694325464374376776313457785622361119232589082131818578591461837297",
            10
        ).unwrap()
    }
    
    pub fn n() -> BigUint {
        BigUint::parse_bytes(b"675048016158598417213", 10).unwrap()
    }
    
    pub fn gx() -> BigUint {
        BigUint::parse_bytes(
            b"18999816458520350299014628291870504329073391058325678653840191278128672378485029664052827205905352913351648904170809",
            10
        ).unwrap()
    }
    
    pub fn gy() -> BigUint {
        BigUint::parse_bytes(
            b"7233699725243644729688547165924232430035643592445942846958231777803539836627943189850381859836033366776176689124317",
            10
        ).unwrap()
    }
    
    pub fn kx() -> BigUint {
        BigUint::parse_bytes(
            b"7147768390112741602848314103078506234267895391544114241891627778383312460777957307647946308927283757886117119137500",
            10
        ).unwrap()
    }
    
    pub fn ky() -> BigUint {
        BigUint::parse_bytes(
            b"20525272195909974311677173484301099561025532568381820845650748498800315498040161314197178524020516408371544778243934",
            10
        ).unwrap()
    }
    
    pub fn priv_key() -> BigUint {
        BigUint::parse_bytes(b"100266970209474387075", 10).unwrap()
    }
}

/// License information parsed from license type string
#[derive(Debug, Clone)]
pub struct LicenseInfo {
    pub chid: u32,
    pub major_ver: u32,
    pub minor_ver: u32,
    pub description: String,
}

impl LicenseInfo {
    pub fn parse(license_type: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = license_type.split('_').collect();
        if parts.len() != 3 {
            anyhow::bail!("License format must be CHID_MAJOR_MINOR (e.g., 029_10_2)");
        }
        
        let chid = parts[0].parse::<u32>()?;
        let major_ver = parts[1].parse::<u32>()?;
        let minor_ver = parts[2].parse::<u32>()?;
        
        let description = LICENSE_TYPES
            .iter()
            .find(|(code, _)| *code == license_type)
            .map(|(_, desc)| desc.to_string())
            .ok_or_else(|| anyhow::anyhow!("Unknown license type"))?;
        
        Ok(Self {
            chid,
            major_ver,
            minor_ver,
            description,
        })
    }
}
