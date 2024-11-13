use std::io;
pub enum State {
    // CLOSED,
    // LISTEN,
    SYNRCVD,
    // SYNSENT,
    ESTAB,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
    ip : etherparse::Ipv4Header,
}
// impl Default for Connection{
//     fn default() -> Self {
//         Self{
//             state: State::LISTEN,
//         }
//     }
// }
/*
```
State of the Send Sequence Space (RFC 793 S3.2 F4)

                   1         2          3          4
              ----------|----------|----------|----------
                     SND.UNA    SND.NXT    SND.UNA
                                          +SND.WND

        1 - old sequence numbers which have been acknowledged
        2 - sequence numbers of unacknowledged data
        3 - sequence numbers allowed for new data transmission
        4 - future sequence numbers which are not yet allowed

                          Send Sequence Space


```

*/
pub struct SendSequenceSpace {
    //send unacknowledged
    una: u32,
    //send next
    nxt: u32,
    // send window
    wnd: u16,
    //send urgent pointer
    up: bool,
    //segment sequence number used for last window update
    wl1: usize,
    //segment acknowledgment number used for last window update
    wl2: usize,
    //initial send sequence number
    iss: u32,
}

/*
```
State of the Receive Sequence Space (RFC 793 S3.2 F4)

1          2          3
----------|----------|----------
   RCV.NXT    RCV.NXT
             +RCV.WND

1 - old sequence numbers which have been acknowledged
2 - sequence numbers allowed for new reception
3 - future sequence numbers which are not yet allowed
```



*/

pub struct RecvSequenceSpace {
    // receive next
    nxt: u32,
    //receive window
    wnd: u16,
    //receive urgent pointer
    up: bool,
    //initil receive sequence number
    irs: u32,
}

impl Connection {
    pub fn accept<'a>(
        nic: &mut tun_tap::Iface,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<Option<Self>> {
        let mut buf = [0u8; 1500];

        if !tcp_header.syn() {
            //only expected SYN packets
            return Ok(None);
        }

        let iss = 0;
        let mut connection: Connection = Connection {
            state: State::SYNRCVD,
            recv: RecvSequenceSpace {
                nxt: tcp_header.sequence_number() + 1,
                wnd: tcp_header.window_size(),
                irs: tcp_header.sequence_number(),
                up: false,
            },
            send: SendSequenceSpace {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0,
            },
            ip :etherparse::Ipv4Header::new(
            0,
            64,
            etherparse::IpNumber::TCP,
            ip_header.destination(),
            ip_header.source(),
        )
        .expect("Failed to create IPv4 Headers"),
        };

        //start establishing a connection
        let mut syn_ack = etherparse::TcpHeader::new(
            tcp_header.destination_port(),
            tcp_header.source_port(),
            connection.send.iss,
            connection.send.wnd,
        );
        syn_ack.acknowledgment_number = connection.recv.nxt;
        syn_ack.ack = true;
        syn_ack.syn = true;
        
        connection.ip.set_payload_len(syn_ack.header_len_u16() as usize +0); 

      

        //kernel takes care of it.
        // syn_ack.checksum = syn_ack.calc_checksum_ipv4(&connection.ip,&[]).expect("Failed to computer checksum");

        eprintln!("got ip header: \n {:02x?}", ip_header);
        eprintln!("got tcp header: \n {:02x?}", tcp_header);

        //wrrite the headers to the buffer
        let unwritten = {
            let mut unwritten = &mut buf[..];

            connection.ip
                .write(&mut unwritten)
                .expect("Failed to write to IP stream");

            syn_ack
                .write(&mut unwritten)
                .expect("Failed to write to TCP stream");

            unwritten.len()
        };

        
        nic.send(&buf[..unwritten])?;

        return Ok(Some(connection));
    }

    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<()> {
       //acceptable ack check
       //  SND.UNA < SEG.ACK =< SND.NXT

       //be aware of wrapping

       let ackn = tcp_header.acknowledgment_number();

      //voilatoin iff  u<n<=a
      if self.send.una < ackn{
        //check for voilation
            if self.send.nxt >= self.send.una && self.send.nxt < ackn {
                return Ok(());
            }else{
                // check if  n is between u and a
                if self.send.nxt >= ackn && self.send.nxt < self.send.una{
                   
                }else{
                    return Ok(());
                }
            }
      }
       match self.state{
        State::SYNRCVD => {
            //
        unimplemented!();
        },
        State::ESTAB =>{
            unimplemented!();
        }
       }
    }
}
// eprintln!("{:?}:{} -> {:?}:{} of {}b",ip_header.source_addr(), tcp_header.source_port(), ip_header.destination_addr(), tcp_header.destination_port(), data.len());
