use std::io::{self, Write, Read, BufRead, BufReader};
use std::time::Duration;
use serialport::{self, Parity, DataBits, FlowControl, StopBits};


fn main() -> Result<(), io::Error> { 
    let ports = serialport::available_ports().expect("No ports found!");
    
    let port = serialport::new("/dev/ttyAMA0", 1200)
        .parity(Parity::Even)
	    .data_bits(DataBits::Seven)
	    .flow_control(FlowControl::None)
        .stop_bits(StopBits::One)
	    .timeout(Duration::from_millis(1000))
	    .open();

    match port {
        Ok(mut port) => {
            let f = BufReader::with_capacity(20, port);

            for line in f.lines() {
                match line {
                    Ok(l) => {
                        println!("--: {}", l);
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (), Err(e) => eprintln!("{:?}", e),
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to open \"blabla\". Error: {}", e);
            ::std::process::exit(1);
        }
    }
}
