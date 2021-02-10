use std::io::{self, BufRead, BufReader};
use std::time::Duration;
use serialport::{self, Parity, DataBits, FlowControl, StopBits};
use pitinfo_parser::parse_line;


fn main() -> Result<(), io::Error> { 
    let port = serialport::new("/dev/ttyAMA0", 1200)
        .parity(Parity::Even)
	    .data_bits(DataBits::Seven)
	    .flow_control(FlowControl::None)
        .stop_bits(StopBits::One)
	    .timeout(Duration::from_millis(1000))
	    .open();

    match port {
        Ok(port) => {
            let f = BufReader::with_capacity(20, port);

            for line in f.lines() {
                match line {
                    Ok(line) => {
                        let result = parse_line(&line);
                        match result {
                            Ok(message) => {
                                println!("Message: {:?}", message);
                            }
                            Err(e) => {
                                eprintln!("Error reading line: '{}': {}", line, e);
                            }
                        }
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
