use std::net::Ipv4Addr;
use std::str::FromStr;

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
            if let Ok(prefix) = u8::from_str(&a[n + 1..]) {
                if let Ok(addr) = Ipv4Addr::from_str(&a[..n]) {
                    return Ipv4Range::try_from((addr, prefix));
                }
            }
            return Err("Invalid IP subnet");
        } else if let Some(n) = a.find('-') {
            if let Ok(iprange_start) = Ipv4Addr::from_str(a[..n].trim()) {
                if let Ok(iprange_end) = Ipv4Addr::from_str(a[n + 1..].trim()) {
                    return Ipv4Range::try_from((iprange_start, iprange_end));
                }
            }
            return Err("Invalid IP range");
        }
        Err("Invalid IP range/subnet")
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
