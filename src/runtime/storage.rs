use std::error::Error;
use std::path::PathBuf;
use std::{fs, str};

use super::prelude::*;
use crate::framework::prelude::*;
use crate::framework::serialization::{ConcreteControls, SerializableControls};

/// When false will use the appropriate OS config dir; when true will store
/// within the Lattice project's controls_cache folder for easy source control.
const STORE_CONTROLS_CACHE_IN_PROJECT: bool = true;

fn controls_storage_path(sketch_name: &str) -> Option<PathBuf> {
    if STORE_CONTROLS_CACHE_IN_PROJECT {
        return Some(
            lattice_project_root()
                .join("storage")
                .join(format!("{}_controls.json", sketch_name)),
        );
    }

    lattice_config_dir().map(|config_dir| {
        config_dir
            .join("Controls")
            .join(format!("{}_controls.json", sketch_name))
    })
}

pub fn save_controls<T: TimingSource + std::fmt::Debug + 'static>(
    sketch_name: &str,
    control_script: &ControlHub<T>,
) -> Result<PathBuf, Box<dyn Error>> {
    let concrete_controls = ConcreteControls {
        controls: control_script
            .ui_controls()
            .unwrap_or_else(|| UiControls::default()),
        midi_controls: control_script
            .midi_controls()
            .unwrap_or_else(|| MidiControls::new()),
        osc_controls: control_script
            .osc_controls()
            .unwrap_or_else(|| OscControls::new()),
    };

    let serializable_controls = SerializableControls::from(concrete_controls);
    let json = serde_json::to_string_pretty(&serializable_controls)?;
    let path = controls_storage_path(sketch_name)
        .ok_or("Could not determine the configuration directory")?;
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }
    fs::write(&path, json)?;
    Ok(path)
}

impl<T: TimingSource + std::fmt::Debug + 'static> ControlHub<T> {
    pub fn load_from_storage(
        &mut self,
        sketch_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        storage::load_controls::<T>(sketch_name, self)
    }
}

pub fn load_controls<T: TimingSource + std::fmt::Debug + 'static>(
    sketch_name: &str,
    control_script: &mut ControlHub<T>,
) -> Result<(), Box<dyn Error>> {
    let path = controls_storage_path(sketch_name)
        .ok_or("Could not determine controls cache directory")?;
    let bytes = fs::read(path)?;
    let json = str::from_utf8(&bytes).ok().map(|s| s.to_owned()).unwrap();

    let sc = serde_json::from_str::<SerializableControls>(&json)?;

    let mut concrete_controls = ConcreteControls {
        controls: control_script
            .ui_controls()
            .unwrap_or_else(|| UiControls::default()),
        midi_controls: control_script
            .midi_controls()
            .unwrap_or_else(|| MidiControls::new()),
        osc_controls: control_script
            .osc_controls()
            .unwrap_or_else(|| OscControls::new()),
    };

    ConcreteControls::merge_serializable_values((sc, &mut concrete_controls));

    concrete_controls
        .controls
        .values()
        .iter()
        .for_each(|(name, value)| {
            control_script.ui_controls.update_value(name, value.clone());
        });

    concrete_controls.midi_controls.values().iter().for_each(
        |(name, value)| {
            control_script.midi_controls.update_value(name, *value);
        },
    );
    concrete_controls
        .osc_controls
        .values()
        .iter()
        .for_each(|(name, value)| {
            control_script.osc_controls.update_value(name, *value);
        });

    Ok(())
}

pub fn delete_stored_controls(sketch_name: &str) -> Result<(), Box<dyn Error>> {
    let path = controls_storage_path(sketch_name)
        .ok_or("Could not determine the configuration directory")?;
    if path.exists() {
        fs::remove_file(path)?;
        info!("Deleted controls for sketch: {}", sketch_name);
    } else {
        warn!("No stored controls found for sketch: {}", sketch_name);
    }
    Ok(())
}
