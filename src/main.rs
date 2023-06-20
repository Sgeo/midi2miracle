use std::io::Read;

use midly::live::{LiveEvent, self};

fn gmFromMiracleProgram(program: midly::num::u7) -> midly::num::u7 {
    // Attempt to change Miracle instrument number to General MIDI instrument number
    match program.into() {
        0 => 0.into(),
        1 => 0.into(), // Detuned piano?
        2 => 0.into(), // FM Piano?
        3 => 0.into(), // Dyno?
        4 => 6.into(), // Harpsichord
        5 => 7.into(), // Clavinet
        6 => 16.into(), // Unspecified? organ
        7 => 19.into(), // Pipe organ (Church organ?)
        8 => 25.into(), // Steel guitar
        9 => 24.into(), // 12 String Guitar -> Acoustic Guitar (nylon)
        10 => 27.into(), // Guitar => Electric Guitar (clean)
        11 => 105.into(), // Banjo
        12 => 12.into(), // Mandolin -> Marimba, couldn't find Mandolin
        13 => 107.into(), // Koto -> Koto
        14 => 26.into(), // Jazz Guitar -> Electric Guitar (jazz)
        15 => 27.into(), // Clean guitar
        16 => 31.into(), // Chorus 
        x => x.into()
    }
}

fn gmFromMiracleLiveEvent(live_event: &mut midly::live::LiveEvent) {
    if let midly::live::LiveEvent::Midi { channel, message } = live_event {
        if let midly::MidiMessage::ProgramChange { program } = message {
            *program = gmFromMiracleProgram(*program);
        }
    }
}

fn processStream(miracle_output_connection: &mut midir::MidiOutputConnection, stream: &mut std::net::TcpStream) {
    let mut midi_stream = midly::stream::MidiStream::new();
    let mut out_buffer = Vec::<u8>::new();
    loop {
        let mut buffer: [u8; 1] = [0x00; 1];
        stream.read_exact(&mut buffer);
        midi_stream.feed(&buffer, |mut live_event| {
            out_buffer.clear();
            gmFromMiracleLiveEvent(&mut live_event);
            println!("Event: {:?}", &live_event);
            live_event.write(&mut out_buffer);
            miracle_output_connection.send(&out_buffer);
        });
    }
}

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


    let listener_result = std::net::TcpListener::bind("0.0.0.0:5858");
    match listener_result {
        Ok(listener) => {
            println!("Listening in server mode on port 5858. Configure DOSBox to point here as a nullmodem.");
            println!("serial 1 nullmodem server:127.0.0.1 port:5858 transparent:1");
            for stream in listener.incoming() {
                println!("Accepted a connection.");
                processStream(&mut miracle_output_connection, &mut stream?);
            }
        },
        Err(_) => {
            println!("Unable to run server on port 5858. Attempting client mode.");
            let mut stream = std::net::TcpStream::connect("127.0.0.1:5858")?;
            processStream(&mut miracle_output_connection, &mut stream);
        }
    }

    Ok(())

}
