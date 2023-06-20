use std::io::{Read, Write};

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

fn process_output_stream(miracle_output_connection: &mut midir::MidiOutputConnection, stream: &mut std::net::TcpStream) {
    let mut midi_stream = midly::stream::MidiStream::new();
    let mut out_buffer = Vec::<u8>::new();
    stream.set_nodelay(true);
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

fn process_input_stream(_timestamp: u64, midi_data: &[u8], stream: &mut std::net::TcpStream) {
    println!("{:x?}", midi_data);
    stream.write_all(midi_data);
    stream.flush();
}

fn main() -> anyhow::Result<()> {
    let miracle_output = midir::MidiOutput::new("Miracle")?;
    let miracle_output_ports = miracle_output.ports();
    println!("Output ports:");
    for (index, port) in miracle_output_ports.iter().enumerate() {
        println!("{}: {}", index, miracle_output.port_name(&port)?);
    }

    if miracle_output_ports.len() < 1 {
        eprintln!("No output ports!");
        return Ok(())
    }

    print!("Choose an output port: ");
    std::io::stdout().flush()?;

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let output_index: usize = line.trim().parse()?;


    let mut miracle_output_connection = miracle_output.connect(&miracle_output_ports[output_index], "Miracle")?;

    let miracle_input = midir::MidiInput::new("Miracle")?;
    let miracle_input_ports = miracle_input.ports();
    println!("Input ports:");
    for (index, port) in miracle_input_ports.iter().enumerate() {
        println!("{}: {}", index, miracle_input.port_name(&port)?);
    }

    print!("Choose an input port: ");
    std::io::stdout().flush()?;

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let input_index: usize = line.trim().parse()?;


    let mut stream_result = std::net::TcpStream::connect("127.0.0.1:5858");
    let _miracle_input_connection; // Keep input connection alive
    match stream_result {
        Ok(mut stream) => {
            println!("Connected as a client on port 5858.");
            stream.set_nodelay(true)?;
            let mut stream_output = stream.try_clone()?;
            process_output_stream(&mut miracle_output_connection, &mut stream);
            std::thread::spawn(move || {
                process_output_stream(&mut miracle_output_connection, &mut stream_output);
            });
            _miracle_input_connection = miracle_input.connect(&miracle_input_ports[input_index], "Miracle", process_input_stream, stream)?;
        },
        Err(_) => {
            println!("Unable to connect as client on port 5858. Attempting server mode.");
            let mut listener = std::net::TcpListener::bind("127.0.0.1:5858")?;
            println!("Listening in one-off server mode on port 5858. Configure DOSBox to point here as a nullmodem.");
            println!("serial 1 nullmodem server:127.0.0.1 port:5858 transparent:1");

            let (stream, _addr) = listener.accept()?;
            println!("Accepted a connection.");
            stream.set_nodelay(true)?;
            let mut stream_output = stream.try_clone()?;
            std::thread::spawn(move || {
                process_output_stream(&mut miracle_output_connection, &mut stream_output);
            });
            _miracle_input_connection = miracle_input.connect(&miracle_input_ports[input_index], "Miracle", process_input_stream, stream)?;
        }
    }

    println!("Waiting forever...");
    loop {}

    Ok(())

}
