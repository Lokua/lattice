use serde::{Deserialize, Serialize};

use super::map_mode::{MapMode, Mappings};
use crate::framework::control::control_hub::Snapshots;
use crate::framework::prelude::*;

pub const GLOBAL_SETTINGS_VERSION: &str = "1";

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct GlobalSettings {
    pub version: String,
    pub audio_device_name: String,
    pub hrcc: bool,
    pub midi_clock_port: String,
    pub midi_control_in_port: String,
    pub midi_control_out_port: String,
    pub osc_port: u16,
    pub transition_time: f32,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            version: GLOBAL_SETTINGS_VERSION.to_string(),
            audio_device_name: global::audio_device_name(),
            hrcc: false,
            midi_clock_port: global::midi_clock_port(),
            midi_control_in_port: global::midi_control_in_port(),
            midi_control_out_port: global::midi_control_out_port(),
            osc_port: global::osc_port(),
            transition_time: 4.0,
        }
    }
}

pub const PROGRAM_STATE_VERSION: &str = "2";

/// Everything needed to recall a patch
#[derive(Deserialize, Serialize)]
pub struct SerializableProgramState {
    pub version: String,

    // Backwards compat files before "ui_controls" rename
    #[serde(rename = "ui_controls", alias = "controls")]
    pub ui_controls: Vec<ControlConfig>,

    pub midi_controls: Vec<BasicNameValueConfig>,
    pub osc_controls: Vec<BasicNameValueConfig>,

    // Backwards compat files that don't have snapshots field
    #[serde(default)]
    pub snapshots: HashMap<String, SerializableSnapshot>,

    #[serde(default)]
    pub mappings: Mappings,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableSnapshot {
    #[serde(rename = "ui_controls", alias = "controls")]
    pub ui_controls: Vec<ControlConfig>,
    pub midi_controls: Vec<BasicNameValueConfig>,
    pub osc_controls: Vec<BasicNameValueConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct ControlConfig {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    #[serde(with = "control_value_format")]
    pub value: ControlValue,
}

#[derive(Serialize, Deserialize)]
pub struct BasicNameValueConfig {
    pub name: String,
    pub value: f32,
}

mod control_value_format {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(
        value: &ControlValue,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(f) = value.as_float() {
            return serializer.serialize_f32(f);
        }
        if let Some(s) = value.as_string() {
            return serializer.serialize_str(s);
        }
        if let Some(b) = value.as_bool() {
            return serializer.serialize_bool(b);
        }

        serializer.serialize_f32(0.0)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<ControlValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            Float(f32),
            String(String),
            Bool(bool),
        }

        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Float(f) => Ok(ControlValue::from(f)),
            Value::String(s) => Ok(ControlValue::from(s)),
            Value::Bool(b) => Ok(ControlValue::from(b)),
        }
    }
}

impl From<&SaveableProgramState> for SerializableProgramState {
    fn from(state: &SaveableProgramState) -> Self {
        let controls = state
            .ui_controls
            .configs()
            .iter()
            .filter_map(|c| {
                if c.is_separator() {
                    None
                } else {
                    let value = state.ui_controls.values().get(c.name());
                    Some(ControlConfig {
                        kind: c.variant_string(),
                        name: c.name().to_string(),
                        value: value.unwrap_or(&c.value()).clone(),
                    })
                }
            })
            .collect();

        let midi_controls = state
            .midi_controls
            .values()
            .iter()
            .map(|(name, value)| BasicNameValueConfig {
                name: name.clone(),
                value: *value,
            })
            .collect();

        let osc_controls = state
            .osc_controls
            .values()
            .iter()
            .map(|(name, value)| BasicNameValueConfig {
                name: name.clone(),
                value: *value,
            })
            .collect();

        let snapshots = state
            .snapshots
            .iter()
            .map(|(name, snapshot)| {
                (name.clone(), create_serializable_snapshot(state, snapshot))
            })
            .collect();

        let mappings = state.mappings.clone();

        Self {
            version: PROGRAM_STATE_VERSION.to_string(),
            ui_controls: controls,
            midi_controls,
            osc_controls,
            snapshots,
            mappings,
        }
    }
}

fn create_serializable_snapshot(
    state: &SaveableProgramState,
    snapshot: &HashMap<String, ControlValue>,
) -> SerializableSnapshot {
    let mut controls = Vec::new();
    for (name, value) in snapshot {
        if let Some(config) = state.ui_controls.config(name) {
            controls.push(ControlConfig {
                kind: config.variant_string(),
                name: name.clone(),
                value: value.clone(),
            });
        }
    }

    let mut midi_controls = Vec::new();
    for (name, value) in snapshot {
        if state.midi_controls.has(name) {
            midi_controls.push(BasicNameValueConfig {
                name: name.clone(),
                value: value.as_float().unwrap(),
            });
        }
    }

    let mut osc_controls = Vec::new();
    for (name, value) in snapshot {
        if state.osc_controls.has(name) {
            osc_controls.push(BasicNameValueConfig {
                name: name.clone(),
                value: value.as_float().unwrap(),
            });
        }
    }

    SerializableSnapshot {
        ui_controls: controls,
        midi_controls,
        osc_controls,
    }
}

/// Intermediary structure used to transfer program state to and from
/// program/serialization contexts
#[derive(Debug)]
pub struct SaveableProgramState {
    pub ui_controls: UiControls,
    pub midi_controls: MidiControls,
    pub osc_controls: OscControls,
    pub snapshots: Snapshots,
    pub mappings: Mappings,
}

impl Default for SaveableProgramState {
    fn default() -> Self {
        Self {
            ui_controls: UiControlBuilder::new().build(),
            midi_controls: MidiControlBuilder::new().build(),
            osc_controls: OscControlBuilder::new().build(),
            snapshots: HashMap::default(),
            mappings: HashMap::default(),
        }
    }
}

impl SaveableProgramState {
    /// Merge incoming serialized data into self
    pub fn merge(&mut self, serialized_state: SerializableProgramState) {
        self.merge_ui_controls(&serialized_state);
        self.mappings = serialized_state.mappings.clone();

        // Must happen before merging MIDI controls otherwise there will be no
        // MIDI proxy configs to merge the saved MIDI proxy values into
        self.setup_midi_mappings();

        self.merge_midi_controls(&serialized_state);
        self.merge_osc_controls(&serialized_state);

        // Note: this consumes serialized_state due to snapshots ownership
        // transfer
        self.merge_snapshots(serialized_state);
    }

    fn merge_ui_controls(
        &mut self,
        serialized_state: &SerializableProgramState,
    ) {
        self.ui_controls
            .values_mut()
            .iter_mut()
            .for_each(|(name, value)| {
                let s = serialized_state
                    .ui_controls
                    .iter()
                    .find(|s| s.name == *name)
                    .map(|s| s.value.clone());

                if let Some(s) = s {
                    *value = s;
                }
            });
    }

    fn setup_midi_mappings(&mut self) {
        self.mappings.iter().for_each(|(name, (ch, cc))| {
            let (min, max) = self.ui_controls.slider_range(name);
            self.midi_controls.add(
                &MapMode::proxy_name(name),
                MidiControlConfig {
                    channel: *ch,
                    cc: *cc,
                    min,
                    max,
                    default: 0.0,
                },
            );
        });
    }

    fn merge_midi_controls(
        &mut self,
        serialized_state: &SerializableProgramState,
    ) {
        self.midi_controls
            .configs()
            .iter_mut()
            .for_each(|(name, _)| {
                let value = serialized_state
                    .midi_controls
                    .iter()
                    .find(|s| s.name == *name)
                    .map(|s| s.value);

                if let Some(v) = value {
                    self.midi_controls.update_value(name, v);
                }
            });
    }

    fn merge_osc_controls(
        &mut self,
        serialized_state: &SerializableProgramState,
    ) {
        self.osc_controls.with_values_mut(|values| {
            values.iter_mut().for_each(|(name, value)| {
                let s = serialized_state
                    .osc_controls
                    .iter()
                    .find(|s| s.name == *name)
                    .map(|s| s.value);

                if let Some(s) = s {
                    *value = s
                }
            });
        });
    }

    fn merge_snapshots(&mut self, serialized_state: SerializableProgramState) {
        self.snapshots.clear();

        for (name, snapshot) in serialized_state.snapshots {
            let mut values = HashMap::default();

            for control in &snapshot.ui_controls {
                values.insert(control.name.clone(), control.value.clone());
            }

            for midi_control in &snapshot.midi_controls {
                values.insert(
                    midi_control.name.clone(),
                    ControlValue::from(midi_control.value),
                );
            }

            for osc_control in &snapshot.osc_controls {
                values.insert(
                    osc_control.name.clone(),
                    ControlValue::from(osc_control.value),
                );
            }

            self.snapshots.insert(name, values);
        }
    }
}
