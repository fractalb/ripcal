use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const HELP_STR: &str = "ripcal -h or ripcal --help\n\t\
                               displays this help\n\
                        ripcal --version\n\t\
                               display the program version\n\
                        ripcal <ip-address>...\n\t\
                               Converts each <ip-address> to corresponding integer";

fn print_version() {
  println!("{} - {}", PKG_NAME, VERSION);
}

fn print_help() {
  println!("{}", HELP_STR);
}

/** ripcal <ip-address>...
 * Converts each <ip-address> from
 * dotted quad => hexadecimal
 * hexadecimal => dotted quad
 * decimal     => dotted quad
 */
fn main() {
  let mut itr =  env::args();
  // println!("Program name: {:?}", itr.next().unwrap());
  itr.next(); // Skip program name.

  let mut argc = 0u32;
  for a in itr {
    argc += 1;
    if argc == 1 {
      if a == "--version" {
        print_version();
        return;
      } else if a == "--help" || a == "-h" {
        print_help();
        return;
      }
    }
    if let Ok(addr) = Ipv4Addr::from_str(&a) {
      // Dotted quad IPv4 address
      let ip : u32 = addr.into();
      println!("{} = {:#x}", a, ip);
    } else if let Some(a2) = a.strip_prefix("0x") {
      // A hexadecimal number as IPv4 address
      if let Ok(ip) = u32::from_str_radix(&a2, 16) {
        let addr = Ipv4Addr::from(ip);
        println!("{} = {}", a, addr);
      } else {
        println!("Invaid IP address: {}", a);
      }
    } else if let Ok(ip) = a.parse::<u32>() {
      // A decimal number as IPv4 address
      let addr = Ipv4Addr::from(ip);
      println!("{} = {}", a, addr);
    } else {
      println!("Invaid IP address: {}", a);
    }
  }
}
