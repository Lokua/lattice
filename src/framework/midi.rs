use midir::Ignore;
use midir::MidiInput;
use midir::MidiOutput;
use std::error::Error;
use std::thread;
use std::time::Duration;

use super::prelude::*;

pub const INPUT_PORT_NAME: &str = "IAC Driver Lattice In";

pub fn on_message<F>(callback: F) -> Result<(), Box<dyn Error>>
where
    F: Fn(&[u8]) + Send + Sync + 'static,
{
    let midi_in = MidiInput::new("Lattice Shared Input")?;

    let in_ports = midi_in.ports();
    let in_port = in_ports
        .iter()
        .find(|p| midi_in.port_name(p).unwrap_or_default() == INPUT_PORT_NAME)
        .expect("Unable to find input port")
        .clone();

    info!("Connecting to {}", INPUT_PORT_NAME);

    thread::spawn(move || {
        // _conn_in needs to be a named parameter,
        // because it needs to be kept alive until the end of the scope
        let _conn_in = midi_in
            .connect(
                &in_port,
                "Lattice Shared Input Read",
                move |_stamp, message, _| {
                    trace!("MIDI message: {:?}", message);
                    callback(message);
                },
                (),
            )
            .expect("Unable to connect");

        info!("Connected to {}", INPUT_PORT_NAME);

        loop {
            thread::sleep(Duration::from_millis(100));
        }
    });

    Ok(())
}

pub fn print_ports() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir test input")?;
    midi_in.ignore(Ignore::None);
    let midi_out = MidiOutput::new("midir test output")?;

    info!("\nAvailable input ports:");
    for (i, p) in midi_in.ports().iter().enumerate() {
        info!("{}: {}", i, midi_in.port_name(p)?);
    }

    info!("\nAvailable output ports:");
    for (i, p) in midi_out.ports().iter().enumerate() {
        info!("{}: {}", i, midi_out.port_name(p)?);
    }

    Ok(())
}
