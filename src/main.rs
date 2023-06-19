use std::io::Read;

fn main() -> anyhow::Result<()> {
    let miracle_output = midir::MidiOutput::new("Miracle")?;
    let miracle_output_ports = miracle_output.ports();
    for port in &miracle_output_ports {
        println!("Detected output port {}", miracle_output.port_name(&port)?);
    }

    if miracle_output_ports.len() < 1 {
        eprintln!("No ports!");
        return Ok(())
    }

    let mut miracle_output_connection = miracle_output.connect(&miracle_output_ports[0], "Miracle")?;

    let listener = std::net::TcpListener::bind("127.0.0.1:5858")?;
    println!("Listening on port 5858. Configure DOSBox to point here as a nullmodem.");
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut midi_stream = midly::stream::MidiStream::new();
        let mut out_buffer = Vec::<u8>::new();
        loop {
            let mut buffer: [u8; 1] = [0x00; 1];
            // stream.read_exact(&mut buffer[0..1])?;
            // let data_byte_count = match buffer[0] & 0xF0 {
            //     0x90 => 2,
            //     0xB0 => 2,
            //     0xC0 => 1,
            //     _ => 0
            // };
            // if data_byte_count == 0 {
            //     eprintln!("Unrecognized control byte!");
            //     return Ok(());
            // }
            // stream.read_exact(&mut buffer[1..data_byte_count+1])?;
            stream.read_exact(&mut buffer)?;
            midi_stream.feed(&buffer, |live_event| {
                out_buffer.clear();
                live_event.write(&mut out_buffer);
                miracle_output_connection.send(&out_buffer);
            });

        }
    }

    Ok(())
}
