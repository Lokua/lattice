use std::any::Any;

use crate::framework::prelude::*;

/// Type erasure trait that enables object-safe access to generic
/// `ControlHub<T>` instances. This enables boxed sketches to expose their
/// controls to the UI and runtime systems without breaking object safety rules.
/// It serves as the interface layer between the type-parameterized
/// `ControlHub<T>` and the object-safe `SketchAll` trait used in the
/// registry.
pub trait ControlProvider {
    fn ui_controls(&self) -> Option<UiControls>;
    fn ui_controls_mut(&mut self) -> &mut UiControls;
    fn ui_control_configs(&self) -> &Vec<Control>;
    fn ui_control_configs_mut(&mut self) -> &mut Vec<Control>;
    fn update_ui_value(&mut self, name: &str, value: ControlValue);
    fn midi_controls(&self) -> Option<MidiControls>;
    fn osc_controls(&self) -> Option<OscControls>;
    fn take_snapshot(&mut self, id: &str);
    fn recall_snapshot(&mut self, id: &str) -> Result<(), String>;
    fn delete_snapshot(&mut self, id: &str);
    fn clear_snapshots(&mut self);
    fn set_transition_time(&mut self, transition_time: f32);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: TimingSource + 'static> ControlProvider for ControlHub<T> {
    fn ui_controls(&self) -> Option<UiControls> {
        Some(self.ui_controls.clone())
    }

    fn ui_controls_mut(&mut self) -> &mut UiControls {
        &mut self.ui_controls
    }

    fn ui_control_configs(&self) -> &Vec<Control> {
        self.ui_controls.configs()
    }

    fn ui_control_configs_mut(&mut self) -> &mut Vec<Control> {
        self.ui_controls.configs_mut()
    }

    fn update_ui_value(&mut self, name: &str, value: ControlValue) {
        self.ui_controls.update_value(name, value)
    }

    fn midi_controls(&self) -> Option<MidiControls> {
        Some(self.midi_controls.clone())
    }

    fn osc_controls(&self) -> Option<OscControls> {
        Some(self.osc_controls.clone())
    }

    fn take_snapshot(&mut self, id: &str) {
        self.take_snapshot(id);
    }

    fn recall_snapshot(&mut self, id: &str) -> Result<(), String> {
        self.recall_snapshot(id)
    }

    fn delete_snapshot(&mut self, id: &str) {
        self.delete_snapshot(id);
    }

    fn clear_snapshots(&mut self) {
        self.clear_snapshots();
    }

    fn set_transition_time(&mut self, transition_time: f32) {
        self.set_transition_time(transition_time);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
