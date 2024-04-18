use std::env;
use std::io::stdin;
use std::io::BufRead;
use std::iter::Iterator;
use std::net::Ipv4Addr;
use std::str::FromStr;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const HELP_STR: &str = "ripcal [-i | -x | -q ] [-r] [<ip-address>...]\n\t\
                               Converts each <ip-address> to different formats\n\t\
                               Options:\n\t\
                               --integer or -i\n\t\t\
                                       Converts to a deca-decimal integer\n\t\
                               --hex or -x\n\t\t\
                                       Converts to a hexa-decimal integer\n\t\
                               --ipv4 or -q\n\t\t\
                                       Converts to a ip-quad\n\t\
                               --reverse-bytes or -r\n\t\t\
                                       Reverse the byte order\n\n\t\
                               If no ip-address arguments are given, then it'll\n\t\
                               read from stdin and output to stdout (filter mode)\n\n\
                        \
                        ripcal <ip-addr/subnet> | \"<ip-start - ip-end>\"\n\t\
                                ip-addr/subnet will be converted to the corresponding\n\t\
                                ip-range (\"start - end\"). \"start - end\" (ip-range)\n\t\
                                will be converted to the minimal ip-addr/subnet which\n\t\
                                covers the given range.\n\n\
                        \
                        ripcal -h or ripcal --help\n\t\
                               displays this help\n\n\
                        \
                        ripcal --version\n\t\
                               displays the program version\n";

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

fn format_iprange(iprange_start: u32, iprange_end: u32) -> String {
    return format!(
        "{} - {}",
        Ipv4Addr::from(iprange_start),
        Ipv4Addr::from(iprange_end)
    );
}

fn format_ipsubnet_as_iprange(ipaddr: u32, prefix: u8) -> String {
    let iprange_start: u32 = mask_ip_addr(ipaddr, prefix);
    let iprange_end: u32 = iprange_start | !mask_from_prefix(prefix);
    return format_iprange(iprange_start, iprange_end);
}

fn format_ipsubnet(ipaddr: u32, prefix: u8) -> String {
    let prefix: u8 = if prefix > 32 { 32 } else { prefix };
    return format!(
        "{}/{}",
        Ipv4Addr::from(mask_ip_addr(ipaddr, prefix)),
        prefix
    );
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
    for a in itr {
        if a == "--reverse-bytes" || a == "-r" {
            config.reverse_bytes = true;
        } else if a == "--integer" || a == "-i" {
            config.conversion_type = ConversionType::DecaDecimal;
        } else if a == "--hex" || a == "-x" {
            config.conversion_type = ConversionType::HexaDecimal;
        } else if a == "--ipv4" || a == "-q" {
            config.conversion_type = ConversionType::IpQuad;
        } else {
            empty_optional_args = false;
            process_ipaddress(&a, &config);
        }
    }

    // Enter filter mode.
    // Read from stdin and print to stdout
    if empty_optional_args {
        process_stdin(config);
    }
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

fn process_ipaddress(a: &str, config: &Config) -> () {
    if let Some(n) = a.find('/') {
        if let Ok(prefix) = u8::from_str(&a[n + 1..]) {
            if let Ok(addr) = Ipv4Addr::from_str(&a[..n]) {
                let addr: u32 = addr.into();
                let output = format_ipsubnet(addr, prefix);
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
                let output = format_ipsubnet(iprange_start, prefix);
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
