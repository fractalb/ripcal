use std::env;
use std::io::stdin;
use std::io::BufRead;
use std::iter::Iterator;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::vec::Vec;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const HELP_STR: &str = r#"ripcal [-i | -x | -q ] [-r] [<ip-address>...]
        Converts each <ip-address> to different formats
        Options:
        --integer or -i
                Converts to a deca-decimal integer
        --hex or -x
                Converts to a hexa-decimal integer
        --ipv4 or -q
                Converts to a ip-quad
        --reverse-bytes or -r\
                Reverse the byte order

        If no ip-address arguments are given, then it'll
        read from stdin and output to stdout (filter mode)

ripcal <ip-addr/subnet> | \"<ip-start - ip-end>\"
        ip-addr/subnet will be converted to the corresponding
        ip-range (\"start - end\"). \"start - end\" (ip-range)
        will be converted to the minimal ip-addr/subnet which
        covers the given range.

ripcal -m (<ip-addr/subnet> | <ip-range>)...
        Merges all the ranges/subnets and presents a minimal
        set of ranges and subnets that exactly covers the
        specified subnets/ranges on the command line.

ripcal -h or ripcal --help
        displays this help

ripcal --version
        displays the program version"#;

fn print_version() -> () {
    println!("{} - {}", PKG_NAME, VERSION);
}

fn print_help() {
    println!("{}", HELP_STR);
}

#[derive(Copy, Clone)]
enum InputType {
    DecaDecimal = 1,
    HexaDecimal = 2,
    IpQuad = 3,
}

type OutputType = InputType;

struct Config {
    reverse_bytes: bool,
    filter_mode: bool,
    output_type: Option<OutputType>,
}

impl Config {
    fn default_config() -> Config {
        Config {
            reverse_bytes: false,
            filter_mode: false,
            output_type: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Ipv4Range {
    start: Ipv4Addr,
    end: Ipv4Addr,
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

impl Ipv4Range {
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

    fn parse_range(a: &str) -> Result<Ipv4Range, &'static str> {
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
        Err("Invalid range/subnet")
    }
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Ipv4Subnet {
    addr: Ipv4Addr,
    prefix: u8,
}

impl Ipv4Subnet {
    fn start_addr(self: &Self) -> Ipv4Addr {
        mask_ip_addr(self.addr, self.prefix)
    }
    fn end_addr(self: &Self) -> Ipv4Addr {
        let start = mask_ip_addr(self.addr, self.prefix);
        &start | Ipv4Addr::from(!make_mask(self.prefix))
    }
}

impl std::convert::TryFrom<(Ipv4Addr, u8)> for Ipv4Subnet {
    type Error = &'static str;
    fn try_from(t: (Ipv4Addr, u8)) -> Result<Self, <Self as TryFrom<(Ipv4Addr, u8)>>::Error> {
        if t.1 > 32 {
            Err("Invalid prefix")
        } else {
            Ok(Self {
                addr: t.0,
                prefix: t.1,
            })
        }
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

/**
 * Expects a sorted ranges, in non-decreasing order.
 */
fn merge_ranges(ranges: &mut Vec<Ipv4Range>) {
    let n = ranges.len();
    if n < 2 {
        return;
    }

    let mut j = 0;
    for i in 1..n {
        // It merges not only overlapping subnets, but also
        // adjacent subnets.
        // eg: 192.168.24.2.2/32, 192.168.24.2.3/32 => 192.168.24.2.2/31
        let start: u32 = ranges[i].start.into();
        let end: u32 = ranges[j].end.into();
        if start > end + 1 {
            j += 1;
            ranges[j] = ranges[i];
        } else {
            ranges[j].end = std::cmp::max(ranges[j].end, ranges[i].end);
        }
    }
    j += 1;
    ranges.drain(j..);
}

fn get_output_type(input_type: InputType, output_type: Option<OutputType>) -> OutputType {
    match output_type {
        Some(contype) => contype,
        None => match input_type {
            InputType::IpQuad => OutputType::HexaDecimal,
            _ => OutputType::IpQuad,
        },
    }
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

fn mask_ip_addr(ip: Ipv4Addr, prefix: u8) -> Ipv4Addr {
    return ip & Ipv4Addr::from(make_mask(prefix));
}

fn ipaddr_to_string(ipaddr: Ipv4Addr, output_type: OutputType, reverse_bytes: bool) -> String {
    let ip: u32 = ipaddr.into();
    let ip: u32 = if reverse_bytes { ip.swap_bytes() } else { ip };
    match output_type {
        OutputType::DecaDecimal => format!("{}", ip),
        OutputType::HexaDecimal => format!("{:#x}", ip),
        OutputType::IpQuad => format!("{}", Ipv4Addr::from(ip)),
    }
}

/** ripcal <ip-address>...
 * Converts each <ip-address> from
 * dotted quad => hexadecimal
 * hexadecimal => dotted quad
 * decimal     => dotted quad
 */
fn main() {
    let mut itr = env::args();
    // println!("Program name: {:?}", itr.next().unwrap());
    itr.next(); // Skip program name.

    if let Some(a) = itr.next() {
        if a == "--version" {
            return print_version();
        } else if a == "--help" || a == "-h" {
            return print_help();
        } else {
            process_args(&mut env::args())
        }
    } else {
        // Enter filter mode.
        // Read from stdin and print to stdout
        process_stdin(Config::default_config());
    }
}

fn process_args(itr: &mut std::env::Args) -> () {
    let mut config = Config::default_config();
    itr.next(); // Skip program name.
    let mut no_args = true;
    let mut vec = Vec::<Ipv4Range>::new();
    let mut range_merge = false;
    for a in itr {
        if a == "--reverse-bytes" || a == "-r" {
            config.reverse_bytes = true;
        } else if a == "--integer" || a == "-i" {
            config.output_type = Some(OutputType::DecaDecimal);
        } else if a == "--hex" || a == "-x" {
            config.output_type = Some(OutputType::HexaDecimal);
        } else if a == "--ipv4" || a == "-q" {
            config.output_type = Some(OutputType::IpQuad);
        } else if a == "--merge-ranges" || a == "-m" {
            range_merge = true;
        } else {
            no_args = false;
            if range_merge {
                if let Ok(range) = Ipv4Range::parse_range(&a) {
                    vec.push(range);
                    continue;
                }
                process_ranges(&mut vec);
                vec.clear();
            }
            process_ipaddress(&a, &config);
        }
    }

    // Enter filter mode.
    // Read from stdin and print to stdout
    if no_args {
        process_stdin(config);
    }

    if range_merge {
        process_ranges(&mut vec);
        vec.clear();
    }
}

fn print_range_vec(vec: &Vec<Ipv4Range>) {
    print!("[{}", vec[0]);
    for i in 1..vec.len() {
        print!(", {}", vec[i]);
    }
    println!("]");
}

fn print_subnet_vec(vec: &Vec<Ipv4Subnet>) {
    if vec.is_empty() {
        println!("[]");
        return;
    }
    print!("[{}", vec[0]);
    for i in 1..vec.len() {
        print!(", {}", vec[i]);
    }
    println!("]");
}

fn process_ranges(vec: &mut Vec<Ipv4Range>) -> () {
    if vec.is_empty() {
        return;
    }
    vec.sort();
    merge_ranges(vec);
    print_range_vec(&vec);

    let mut vec2: Vec<Ipv4Subnet> = Vec::new();
    for i in 0..vec.len() {
        let tmp = ip_range_to_subnets(vec[i]);
        vec2.extend(tmp.iter());
    }
    print_subnet_vec(&vec2);
}

fn process_stdin(config: Config) -> () {
    let config = Config {
        filter_mode: true,
        ..config
    };
    let input = stdin();
    for line in input.lock().lines() {
        if let Ok(a) = line {
            if a == "" {
                println!("");
            } else {
                process_ipaddress(&a, &config);
            }
        } else {
            println!("Error reading input");
        }
    }
}

fn count_suffix_zero_bits(ip: u64) -> u8 {
    let mut i = 0;
    let mut ip = ip;
    while (i <= 32) && ((ip & 0x1) == 0x0) {
        i += 1;
        ip >>= 1
    }
    return i;
}

fn ip_range_to_subnets(range: Ipv4Range) -> Vec<Ipv4Subnet> {
    let mut vec: Vec<Ipv4Subnet> = Vec::new();
    let start: u32 = range.start.into();
    let end: u32 = range.end.into();
    let mut start: u64 = start as u64;
    let end: u64 = end as u64;
    while start <= end {
        let mut s: u8 = count_suffix_zero_bits(start);
        let mut diff: u64 = (1u64 << s) - 1;
        while (start + diff) > end {
            diff >>= 1;
            s -= 1;
        }
        vec.push(Ipv4Subnet {
            addr: Ipv4Addr::from(start as u32),
            prefix: 32 - s,
        });
        start += diff + 1;
    }
    return vec;
}

fn process_ipaddress(a: &str, config: &Config) {
    if let Some(n) = a.find('/') {
        // A subnet (eg. 192.168.18.0/24)
        if let Ok(prefix) = u8::from_str(&a[n + 1..]) {
            if let Ok(addr) = Ipv4Addr::from_str(&a[..n]) {
                if let Ok(subnet) = Ipv4Subnet::try_from((addr, prefix)) {
                    let output = format!("{subnet}")
                        + "\n"
                        + &format!("{subnet}")
                        + " = "
                        + &format!("{}", Ipv4Range::from(&subnet));
                    print_output(&output, &a, &config);
                    return;
                }
            }
        }
        println!("Invalid IP subnet: {}", a);
    } else if let Some(n) = a.find('-') {
        // A range (eg. 192.168.18.0-192.168.18.255)
        if let Ok(iprange_start) = Ipv4Addr::from_str(a[..n].trim()) {
            if let Ok(iprange_end) = Ipv4Addr::from_str(a[n + 1..].trim()) {
                if let Ok(iprange) = Ipv4Range::try_from((iprange_start, iprange_end)) {
                    let subnet = Ipv4Subnet::from(&iprange);
                    let output = format!("{subnet}")
                        + "\n"
                        + &format!("{subnet}")
                        + " = "
                        + &format!("{}", Ipv4Range::from(&subnet));
                    print_output(&output, &a, &config);
                    return;
                }
            }
        }
        println!("Invalid IP range: {}", a);
    } else if let Ok(addr) = Ipv4Addr::from_str(&a) {
        // Dotted quad IPv4 address (eg. 192.168.18.0)
        let input_type = InputType::IpQuad;
        let output_type = get_output_type(input_type, config.output_type);
        let output = ipaddr_to_string(addr, output_type, config.reverse_bytes);
        print_output(&output, &a, &config);
    } else if let Ok(ip) = a.parse::<u32>() {
        // A de number that can treated as an IPv4 address
        // A decimal number as IPv4 address
        let addr = Ipv4Addr::from(ip);
        let input_type = InputType::DecaDecimal;
        let output_type = get_output_type(input_type, config.output_type);
        let output = ipaddr_to_string(addr, output_type, config.reverse_bytes);
        print_output(&output, &a, &config);
    } else {
        // See if it's a hexadecimal number as IPv4 address
        let ip;
        if let Some(a2) = a.strip_prefix("0x") {
            // hexadecimal number with "0x" prefix?
            ip = u32::from_str_radix(&a2, 16);
        } else {
            // hexadecimal number without a "0x" prefix?
            ip = u32::from_str_radix(&a, 16);
        }
        if let Ok(ip) = ip {
            let addr = Ipv4Addr::from(ip);
            let input_type = InputType::HexaDecimal;
            let output_type = get_output_type(input_type, config.output_type);
            let output = ipaddr_to_string(addr, output_type, config.reverse_bytes);
            print_output(&output, &a, &config);
            return;
        }
        // Not even a hexadecimal number
        println!("Invalid IP address: {}", a);
    }
}

fn print_output(output: &str, input: &str, config: &Config) -> () {
    if config.filter_mode {
        println!("{}", output);
    } else {
        println!(
            "{}{} = {}",
            if config.reverse_bytes { "Reverse " } else { "" },
            input,
            output
        );
    }
}
