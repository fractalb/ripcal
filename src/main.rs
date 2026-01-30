mod iprange;
use iprange::Ipv4Range;
use iprange::Ipv4Subnet;
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
        let start: u32 = ranges[i].start().into();
        let end: u32 = ranges[j].end().into();
        if start > end + 1 {
            j += 1;
            ranges[j] = ranges[i];
        } else {
            let new_end = std::cmp::max(ranges[j].end(), ranges[i].end());
            ranges[j].update_end(new_end);
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

    let Some(a) = itr.next() else {
        // Enter filter mode.
        // Read from stdin and print to stdout
        return process_stdin(Config::default_config());
    };

    if a == "--version" {
        print_version()
    } else if a == "--help" || a == "-h" {
        print_help()
    } else {
        process_args(&mut env::args())
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
    // This single line should be enough. Needs testing though.
    // print!("Second: [{}]", vec.iter().map(|x|{x.to_string()}).collect::<Vec<String>>().join(", "));
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
        let tmp = vec[i].to_subnets();
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

fn process_ipaddress(a: &str, config: &Config) {
    if let Ok(iprange) = Ipv4Range::from_str(a) {
        let subnet = Ipv4Subnet::from(&iprange);
        let output = format!("{subnet}")
            + "\n"
            + &format!("{subnet}")
            + " = "
            + &format!("{}", Ipv4Range::from(&subnet));
        print_output(&output, &a, &config);
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
