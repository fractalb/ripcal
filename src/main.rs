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
enum ConversionType {
    DefaultConversion = 0,
    DecaDecimal = 1,
    HexaDecimal = 2,
    IpQuad = 3,
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
    conversion_type: ConversionType,
}

impl Config {
    fn default_config() -> Config {
        Config {
            reverse_bytes: false,
            filter_mode: false,
            conversion_type: ConversionType::DefaultConversion,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Ipv4Range {
    start: u32,
    end: u32,
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Ipv4Subnet {
    addr: u32,
    prefix: u8,
}

impl std::fmt::Display for Ipv4Subnet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix)
    }
}

fn merge_2<'a>(new_vec: &'a mut Vec<Ipv4Range>, r2: Ipv4Range) -> &'a mut Vec<Ipv4Range> {
    if new_vec.is_empty() {
        new_vec.push(r2);
        return new_vec;
    }
    let len = new_vec.len();
    let r1: &mut Ipv4Range = &mut new_vec[len - 1];
    if r2.start > r1.end + 1 {
        new_vec.push(r2);
        return new_vec;
    }

    r1.end = std::cmp::max(r1.end, r2.end);
    return new_vec;
}

fn merge_ranges(vec: &Vec<Ipv4Range>) -> Vec<Ipv4Range> {
    //println!("merging: {:?}", vec);
    let mut vec2 = Vec::new();
    if !vec.is_empty() {
        vec2.push(vec[0]);
        for i in 1..vec.len() {
            merge_2(&mut vec2, vec[i]);
        }
    }
    vec2
}

fn get_output_type(input_type: InputType, conversion_type: ConversionType) -> OutputType {
    match conversion_type {
        ConversionType::DecaDecimal => OutputType::DecaDecimal,
        ConversionType::HexaDecimal => OutputType::HexaDecimal,
        ConversionType::IpQuad => OutputType::IpQuad,
        ConversionType::DefaultConversion => match input_type {
            InputType::IpQuad => OutputType::HexaDecimal,
            _ => OutputType::IpQuad,
        },
    }
}

fn mask_from_prefix(prefix: u8) -> u32 {
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

fn mask_ip_addr(ip: u32, prefix: u8) -> u32 {
    return ip & mask_from_prefix(prefix);
}

fn iprange_to_string(range: Ipv4Range) -> String {
    return format!(
        "{} - {}",
        Ipv4Addr::from(range.start),
        Ipv4Addr::from(range.end)
    );
}

fn format_ipsubnet(subnet: Ipv4Subnet) -> String {
    let prefix: u8 = if subnet.prefix > 32 {
        32
    } else {
        subnet.prefix
    };
    return format!("{}/{}", Ipv4Addr::from(subnet.addr), prefix);
}

fn format_ipsubnet_as_iprange(ipaddr: u32, prefix: u8) -> String {
    let range = ip_prefix_to_range(ipaddr, prefix);
    return iprange_to_string(range);
}

fn format_ipaddr(ipaddr: Ipv4Addr, output_type: OutputType, reverse_bytes: bool) -> String {
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
    let mut empty_optional_args = true;
    let mut vec = Vec::<Ipv4Range>::new();
    let mut range_merge = false;
    for a in itr {
        if a == "--reverse-bytes" || a == "-r" {
            config.reverse_bytes = true;
        } else if a == "--integer" || a == "-i" {
            config.conversion_type = ConversionType::DecaDecimal;
        } else if a == "--hex" || a == "-x" {
            config.conversion_type = ConversionType::HexaDecimal;
        } else if a == "--ipv4" || a == "-q" {
            config.conversion_type = ConversionType::IpQuad;
        } else if a == "--merge-ranges" || a == "-m" {
            range_merge = true;
        } else {
            empty_optional_args = false;
            if range_merge {
                if let Some(range) = parse_range(&a) {
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
    if empty_optional_args {
        process_stdin(config);
    }

    if range_merge {
        process_ranges(&mut vec);
        vec.clear();
    }
}

fn parse_range(a: &String) -> Option<Ipv4Range> {
    if let Some(n) = a.find('/') {
        if let Ok(prefix) = u8::from_str(&a[n + 1..]) {
            if let Ok(addr) = Ipv4Addr::from_str(&a[..n]) {
                let addr: u32 = addr.into();
                return Some(ip_prefix_to_range(addr, prefix));
            }
        }
        println!("Invalid IP subnet: {}", a);
    } else if let Some(n) = a.find('-') {
        if let Ok(iprange_start) = Ipv4Addr::from_str(a[..n].trim()) {
            if let Ok(iprange_end) = Ipv4Addr::from_str(a[n + 1..].trim()) {
                let iprange_start: u32 = iprange_start.into();
                let iprange_end: u32 = iprange_end.into();
                if iprange_start > iprange_end {
                    println!("Invalid range: {}", a);
                    return None;
                }
                let range = Ipv4Range {
                    start: iprange_start,
                    end: iprange_end,
                };
                return Some(range);
            }
        }
    }
    return None;
}

fn print_range_vec(vec: &Vec<Ipv4Range>) {
    print!("[{}", iprange_to_string(vec[0]));
    for i in 1..vec.len() {
        print!(", {}", iprange_to_string(vec[i]));
    }
    println!("]");
}

fn print_subnet_vec(vec: &Vec<Ipv4Subnet>) {
    print!("[{}", format_ipsubnet(vec[0]));
    for i in 1..vec.len() {
        print!(", {}", format_ipsubnet(vec[i]));
    }
    println!("]");
}

fn process_ranges(vec: &mut Vec<Ipv4Range>) -> () {
    if vec.is_empty() {
        return;
    }
    vec.sort();
    *vec = merge_ranges(vec);
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

fn get_prefix_from_iprange(start: u32, end: u32) -> u8 {
    for i in 0..32 {
        if (start >> i) == (end >> i) {
            return 32 - i;
        }
    }
    return 0;
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
    let mut start: u64 = range.start.into();
    let end: u64 = range.end.into();
    while start <= end {
        let mut s: u8 = count_suffix_zero_bits(start);
        let mut diff: u64 = (1u64 << s) - 1;
        while (start + diff) > end {
            diff >>= 1;
            s -= 1;
        }
        vec.push(Ipv4Subnet {
            addr: start as u32,
            prefix: 32 - s,
        });
        start += diff + 1;
    }
    return vec;
}

fn ip_prefix_to_range(ip: u32, prefix: u8) -> Ipv4Range {
    let iprange_start: u32 = mask_ip_addr(ip, prefix);
    let iprange_end: u32 = iprange_start | !mask_from_prefix(prefix);
    let range = Ipv4Range {
        start: iprange_start,
        end: iprange_end,
    };
    return range;
}

fn process_ipaddress(a: &str, config: &Config) -> () {
    if let Some(n) = a.find('/') {
        if let Ok(prefix) = u8::from_str(&a[n + 1..]) {
            if let Ok(addr) = Ipv4Addr::from_str(&a[..n]) {
                let addr: u32 = addr.into();
                let subnet = Ipv4Subnet {
                    addr: addr,
                    prefix: prefix,
                };
                let output = format_ipsubnet(subnet);
                let output = output.clone()
                    + "\n"
                    + &output
                    + " = "
                    + &format_ipsubnet_as_iprange(addr, prefix);
                print_output(&output, &a, &config);
                return;
            }
        }
        println!("Invalid IP subnet: {}", a);
    } else if let Some(n) = a.find('-') {
        if let Ok(iprange_start) = Ipv4Addr::from_str(a[..n].trim()) {
            if let Ok(iprange_end) = Ipv4Addr::from_str(a[n + 1..].trim()) {
                let iprange_start: u32 = iprange_start.into();
                let iprange_end: u32 = iprange_end.into();
                if iprange_start > iprange_end {
                    println!("Invalid range: {}", a);
                    return;
                }
                let prefix = get_prefix_from_iprange(iprange_start, iprange_end);
                let subnet = Ipv4Subnet {
                    addr: iprange_start,
                    prefix: prefix,
                };
                let output = format_ipsubnet(subnet);
                let output = output.clone()
                    + "\n"
                    + &output
                    + " = "
                    + &format_ipsubnet_as_iprange(iprange_start, prefix);
                print_output(&output, &a, &config);
                return;
            }
        }
    } else if let Ok(addr) = Ipv4Addr::from_str(&a) {
        // Dotted quad IPv4 address
        let input_type = InputType::IpQuad;
        let output_type = get_output_type(input_type, config.conversion_type);
        let output = format_ipaddr(addr, output_type, config.reverse_bytes);
        print_output(&output, &a, &config);
    } else if let Ok(ip) = a.parse::<u32>() {
        // A decimal number as IPv4 address
        let addr = Ipv4Addr::from(ip);
        let input_type = InputType::DecaDecimal;
        let output_type = get_output_type(input_type, config.conversion_type);
        let output = format_ipaddr(addr, output_type, config.reverse_bytes);
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
            let output_type = get_output_type(input_type, config.conversion_type);
            let output = format_ipaddr(addr, output_type, config.reverse_bytes);
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
