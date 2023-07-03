use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;

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

  for a in itr {
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
