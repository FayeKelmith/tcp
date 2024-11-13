use tun_tap::Iface;
use std::io;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::collections::hash_map::Entry;
mod tcp;


#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy )]
struct Quad{
    src: (Ipv4Addr, u16),
    dest: (Ipv4Addr, u16)
     
}
fn main() -> io::Result<()> {
    let mut connections : HashMap<Quad, tcp::Connection> = Default::default();
   let mut nic = Iface::without_packet_info("tun0", tun_tap::Mode::Tun)?;
   let mut buf = [0u8; 1504];

   loop {
         let nbytes = nic.recv(&mut buf[..])?;
        //  let _eth_flags: u16 = u16::from_be_bytes([buf[0], buf[1]]);
        //  let eth_proto: u16 = u16::from_be_bytes([buf[2], buf[3]]);

         //selecting only ipv4 packets
        //  if eth_proto != 0x0800 {
        //     continue;
        //  }

         match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]){
            Ok(ip_header)=>{
                let source_ip = ip_header.source_addr();
                let destination_ip = ip_header.destination_addr();
                let protocol_number = ip_header.protocol().0;


            //selecting only tcp packets
            if protocol_number != 0x06 {
                continue;
            }

           
            match etherparse::TcpHeaderSlice::from_slice(&buf[  ip_header.slice().len()..nbytes]){
                Ok(tcp_header)=>{

                    let source_port = tcp_header.source_port();
                    let destination_port = tcp_header.destination_port();
                    let packet_size = tcp_header.slice().len();
                    let datai = ip_header.slice().len() + tcp_header.slice().len();
                    match connections.entry(Quad{
                         src: (source_ip, source_port),
                         dest: (destination_ip, destination_port)
                    }){
                        Entry::Occupied(mut c)=>
                        {
                            c.get_mut().on_packet(&mut nic,ip_header, tcp_header, &buf[datai..nbytes ])?;
                        }
                        Entry::Vacant(mut e)=>{
                             if let Some(c) = tcp::Connection::accept(&mut nic,ip_header, tcp_header, &buf[datai..nbytes ])?{
                                e.insert(c);
                             };
                        }
                    }
                    eprintln!("--------------------");
                    eprintln!("{:?}:{} -> {:?}:{}  of {}b",source_ip, source_port, destination_ip, destination_port, packet_size );
                    eprintln!("====================")

                }
                Err(e)=>{
                    eprintln!("Error parsing tcp header: {:?}",e);
                    continue;
                }
            }

          
            }
            Err(e)=>{
                eprintln!("error parsing ipv4 header: {:?}", e);
                continue;
            }
         }
         
   }

}
