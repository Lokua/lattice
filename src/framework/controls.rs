use nannou_egui::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Any sketch using controls needs to implement this trait for the Model
pub trait HasControls {
    fn controls(&mut self) -> &mut Controls;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ControlValue {
    Float(f32),
    Bool(bool),
    String(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Control {
    Slider {
        name: String,
        value: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    Button {
        name: String,
    },
    Checkbox {
        name: String,
        value: bool,
    },
    Select {
        name: String,
        value: String,
        options: Vec<String>,
    },
}

impl Control {
    pub fn name(&self) -> &str {
        match self {
            Control::Slider { name, .. } => name,
            Control::Button { name, .. } => name,
            Control::Checkbox { name, .. } => name,
            Control::Select { name, .. } => name,
        }
    }

    pub fn value(&self) -> ControlValue {
        match self {
            Control::Slider { value, .. } => ControlValue::Float(*value),
            Control::Button { .. } => ControlValue::Bool(false),
            Control::Checkbox { value, .. } => ControlValue::Bool(*value),
            Control::Select { value, .. } => {
                ControlValue::String(value.clone())
            }
        }
    }

    pub fn slider(
        name: &str,
        value: f32,
        range: (f32, f32),
        step: f32,
    ) -> Control {
        Control::Slider {
            name: name.to_string(),
            value,
            min: range.0,
            max: range.1,
            step,
        }
    }
}

pub type ControlValues = HashMap<String, ControlValue>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Controls {
    controls: Vec<Control>,
    values: ControlValues,
}

impl Controls {
    pub fn new(controls: Vec<Control>) -> Self {
        let values = controls
            .iter()
            .map(|control| (control.name().to_string(), control.value()))
            .collect();

        Self { controls, values }
    }

    pub fn get_controls(&self) -> &Vec<Control> {
        &self.controls
    }

    pub fn values(&self) -> &ControlValues {
        &self.values
    }

    pub fn float(&self, name: &str) -> f32 {
        self.check_contains_key(name);
        match self.values.get(name).unwrap() {
            ControlValue::Float(v) => *v,
            _ => panic!("Control '{}' exists but is not a float", name),
        }
    }

    pub fn bool(&self, name: &str) -> bool {
        self.check_contains_key(name);
        match self.values.get(name).unwrap() {
            ControlValue::Bool(v) => *v,
            _ => panic!("Control '{}' exists but is not a bool", name),
        }
    }

    pub fn string(&self, name: &str) -> String {
        self.check_contains_key(name);
        match self.values.get(name).unwrap() {
            ControlValue::String(v) => v.clone(),
            _ => panic!("Control '{}' exists but is not a string", name),
        }
    }

    fn check_contains_key(&self, key: &str) {
        if !self.values.contains_key(key) {
            panic!("Control {} does not exist", key);
        }
    }

    pub fn update_value(&mut self, name: &str, value: ControlValue) {
        self.values.insert(name.to_string(), value);
    }
}

pub fn draw_controls(controls: &mut Controls, ui: &mut egui::Ui) -> bool {
    let controls_list = controls.get_controls().clone();
    let mut any_changed = false;

    for control in controls_list {
        match control {
            Control::Slider {
                name,
                min,
                max,
                step,
                ..
            } => {
                let mut value = controls.float(&name);
                if ui
                    .add(
                        egui::Slider::new(&mut value, min..=max)
                            .text(&name)
                            .step_by(step.into()),
                    )
                    .changed()
                {
                    controls.update_value(&name, ControlValue::Float(value));
                    any_changed = true;
                }
            }
            Control::Checkbox { name, .. } => {
                let mut value = controls.bool(&name);
                if ui.checkbox(&mut value, &name).changed() {
                    controls.update_value(&name, ControlValue::Bool(value));
                    any_changed = true;
                }
            }
            Control::Button { name } => if ui.button(&name).clicked() {},
            Control::Select { name, options, .. } => {
                let mut value = controls.string(&name);
                egui::ComboBox::from_label(&name)
                    .selected_text(&value)
                    .show_ui(ui, |ui| {
                        for option in options {
                            if ui
                                .selectable_value(
                                    &mut value,
                                    option.clone(),
                                    &option,
                                )
                                .changed()
                            {
                                controls.update_value(
                                    &name,
                                    ControlValue::String(value.clone()),
                                );
                                any_changed = true;
                            }
                        }
                    });
            }
        }
    }

    any_changed
}
