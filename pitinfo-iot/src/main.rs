use pitinfo_parser::parse_group;
use serialport::{self, DataBits, FlowControl, Parity, StopBits};
use std::io::{self, BufRead, BufReader};
use std::time::Duration;

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

            for line in f.lines().skip(1) {
                match line {
                    Ok(line) => {
                        // PPOT at the end of the frame gets control chars:
                        // \x03 -> enf of frame, \x02 -> start of frame, and new line
                        let group =
                            String::from(line.trim_end_matches(&['\x03', '\x02', '\x0d'] as &[_]));
                        let result = parse_group(&group);
                        match result {
                            Ok(Some(message)) => {
                                println!("Message: {:<20} -> {:?}", group, message);
                            }
                            Ok(None) => {
                                println!("Message: {:<20} -> Ignored", group);
                            }
                            Err(e) => {
                                eprintln!("Error reading group: '{}': {}", group, e);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
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
