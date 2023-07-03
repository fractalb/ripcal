use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;

fn main() {
  let mut itr =  env::args();
  // println!("Program name: {:?}", itr.next().unwrap());
  itr.next();

  for a in itr {
    if let Ok(addr) = Ipv4Addr::from_str(&a) {
      let ip : u32 = addr.into();
      println!("{} = {:#x}", a, ip);
    } else {
      println!("Invaid IP address: {}", a);
    }
  }
}
