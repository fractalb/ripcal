use std::net::Ipv4Addr;
use std::str::FromStr;
use std::vec::Vec;

fn count_suffix_zero_bits(ip: u64) -> u8 {
    let mut i = 0;
    let mut ip = ip;
    while (i <= 32) && ((ip & 0x1) == 0x0) {
        i += 1;
        ip >>= 1
    }
    return i;
}

fn make_mask(prefix: u8) -> u32 {
    if prefix == 0 {
        return 0;
    }
    let mask: u32 = 0xffffffff;
    if prefix < 32 {
        let n = 32 - prefix;
        return (mask >> n) << n;
    }
    return mask;
}

fn mask_ipaddr(ip: Ipv4Addr, prefix: u8) -> Ipv4Addr {
    return ip & Ipv4Addr::from(make_mask(prefix));
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Ipv4Range {
    start: Ipv4Addr,
    end: Ipv4Addr,
}

impl Ipv4Range {
    pub fn start(self: &Self) -> Ipv4Addr {
        self.start
    }

    pub fn end(self: &Self) -> Ipv4Addr {
        self.end
    }

    /*
     *pub fn update_start(self: &mut Self, start: Ipv4Addr) -> bool {
     *    if start > self.end {
     *        false
     *    } else {
     *        self.start = start;
     *        true
     *    }
     *}
     */
    pub fn update_end(self: &mut Self, end: Ipv4Addr) -> bool {
        if end < self.start {
            false
        } else {
            self.end = end;
            true
        }
    }

    fn get_prefix(self: &Self) -> u8 {
        for i in 0..32 {
            let start: u32 = self.start.into();
            let end: u32 = self.end.into();
            if (start >> i) == (end >> i) {
                return 32 - i;
            }
        }
        return 0;
    }

    pub fn parse_range(a: &str) -> Result<Ipv4Range, &'static str> {
        if let Some(n) = a.find('/') {
            let Ok(prefix) = u8::from_str(&a[n + 1..]) else {
                return Err("Invalid IP subnet prefix");
            };
            let Ok(addr) = Ipv4Addr::from_str(&a[..n]) else {
                return Err("Invalid IP address");
            };
            return Ipv4Range::try_from((addr, prefix));
        } else if let Some(n) = a.find('-') {
            let Ok(iprange_start) = Ipv4Addr::from_str(a[..n].trim()) else {
                return Err("Invalid IP address");
            };
            let Ok(iprange_end) = Ipv4Addr::from_str(a[n + 1..].trim()) else {
                return Err("Invalid IP address");
            };
            return Ipv4Range::try_from((iprange_start, iprange_end));
        }
        Err("Invalid IP range/subnet")
    }

    pub fn to_subnets(self: &Self) -> Vec<Ipv4Subnet> {
        let mut vec: Vec<Ipv4Subnet> = Vec::new();
        let start: u32 = self.start().into();
        let end: u32 = self.end().into();
        let mut start: u64 = start as u64;
        let end: u64 = end as u64;
        while start <= end {
            let mut s: u8 = count_suffix_zero_bits(start);
            let mut diff: u64 = (1u64 << s) - 1;
            while (start + diff) > end {
                diff >>= 1;
                s -= 1;
            }
            vec.push(Ipv4Subnet::try_from((start as u32, 32u8 - s)).unwrap());
            start += diff + 1;
        }
        return vec;
    }
}

impl FromStr for Ipv4Range {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ipv4Range::parse_range(s)
    }
}

impl std::convert::TryFrom<(Ipv4Addr, Ipv4Addr)> for Ipv4Range {
    type Error = &'static str;
    fn try_from(t: (Ipv4Addr, Ipv4Addr)) -> Result<Self, Self::Error> {
        if t.0 > t.1 {
            Err("Invalid Range")
        } else {
            Ok(Ipv4Range {
                start: t.0,
                end: t.1,
            })
        }
    }
}

impl std::convert::TryFrom<(Ipv4Addr, u8)> for Ipv4Range {
    type Error = &'static str;
    fn try_from(t: (Ipv4Addr, u8)) -> Result<Self, Self::Error> {
        Ok(Ipv4Range::from(&Ipv4Subnet::try_from(t)?))
    }
}

impl std::convert::From<&Ipv4Subnet> for Ipv4Range {
    fn from(ipsubnet: &Ipv4Subnet) -> Self {
        Self {
            start: ipsubnet.start_addr(),
            end: ipsubnet.end_addr(),
        }
    }
}

impl std::fmt::Display for Ipv4Range {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} - {}", self.start, self.end)
    }
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Ipv4Subnet {
    addr: Ipv4Addr,
    prefix: u8,
}

impl Ipv4Subnet {
    fn start_addr(self: &Self) -> Ipv4Addr {
        mask_ipaddr(self.addr, self.prefix)
    }
    fn end_addr(self: &Self) -> Ipv4Addr {
        let start = mask_ipaddr(self.addr, self.prefix);
        &start | Ipv4Addr::from(!make_mask(self.prefix))
    }
}

impl FromStr for Ipv4Subnet {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Ipv4Subnet, Self::Err> {
        let Some(n) = s.find('/') else {
            return Err("Invalid subnet string");
        };
        // A subnet (eg. 192.168.18.0/24)
        let Ok(prefix) = u8::from_str(&s[n + 1..]) else {
            return Err("Invalid subnet prefix");
        };
        let Ok(addr) = Ipv4Addr::from_str(&s[..n]) else {
            return Err("Invalid IP address");
        };
        Ipv4Subnet::try_from((addr, prefix))
    }
}

impl std::convert::TryFrom<(Ipv4Addr, u8)> for Ipv4Subnet {
    type Error = &'static str;
    fn try_from(t: (Ipv4Addr, u8)) -> Result<Self, <Self as TryFrom<(Ipv4Addr, u8)>>::Error> {
        if t.1 > 32 {
            Err("Invalid IP subnet prefix")
        } else {
            Ok(Self {
                addr: t.0,
                prefix: t.1,
            })
        }
    }
}

impl std::convert::TryFrom<(u32, u8)> for Ipv4Subnet {
    type Error = &'static str;
    fn try_from(t: (u32, u8)) -> Result<Self, <Self as TryFrom<(u32, u8)>>::Error> {
        Self::try_from((Ipv4Addr::from(t.0), t.1))
    }
}

impl std::convert::From<&Ipv4Range> for Ipv4Subnet {
    fn from(iprange: &Ipv4Range) -> Self {
        Self {
            addr: iprange.start,
            prefix: iprange.get_prefix(),
        }
    }
}

impl std::fmt::Display for Ipv4Subnet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix)
    }
}

#[test]
fn instantiate_types() {
    assert!(Ipv4Range::from_str("192.168.1.0-192.168.0.255").is_err());
    assert!(Ipv4Range::from_str("127.0.0.1").is_err());
    assert_eq!(
        Ipv4Range::from_str("255.255.255.255/32").unwrap(),
        Ipv4Range::from_str("255.255.255.255-255.255.255.255").unwrap()
    );
}

#[test]
fn range_to_subnet_conversion() {
    let r: Ipv4Range = Ipv4Range::from_str("192.168.1.0 - 192.168.1.1").unwrap();
    let s: Ipv4Subnet = Ipv4Subnet::from_str("192.168.1.0/31").unwrap();
    assert_eq!(r.to_subnets(), vec![s]);

    let r: Ipv4Range = Ipv4Range::from_str("0.0.0.0 - 255.255.255.255").unwrap();
    let s: Ipv4Subnet = Ipv4Subnet::from_str("0.0.0.0/0").unwrap();
    assert_eq!(r.to_subnets(), vec![s]);
}
