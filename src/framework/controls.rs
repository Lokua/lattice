use nannou_egui::egui;
use std::collections::HashMap;

/// Any sketch using controls needs to implement this trait for the model
pub trait HasControls {
    fn controls(&mut self) -> &mut Controls;
}

#[derive(Clone)]
pub enum ControlValue {
    Float(f32),
    Bool(bool),
    String(String),
}

#[derive(Clone)]
pub enum Control {
    Slider {
        name: String,
        value: f32,
        min: f32,
        max: f32,
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
}

pub struct Controls {
    controls: Vec<Control>,
    values: HashMap<String, ControlValue>,
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

    pub fn get_float(&self, name: &str) -> f32 {
        match self.values.get(name).unwrap() {
            ControlValue::Float(v) => *v,
            _ => panic!("Control '{}' exists but is not a float", name),
        }
    }

    pub fn get_bool(&self, name: &str) -> bool {
        match self.values.get(name).unwrap() {
            ControlValue::Bool(v) => *v,
            _ => panic!("Control '{}' exists but is not a bool", name),
        }
    }

    pub fn get_string(&self, name: &str) -> String {
        match self.values.get(name).unwrap() {
            ControlValue::String(v) => v.clone(),
            _ => panic!("Control '{}' exists but is not a string", name),
        }
    }

    pub fn update_value(&mut self, name: &str, value: ControlValue) {
        self.values.insert(name.to_string(), value);
    }
}

pub fn draw_controls(controls: &mut Controls, ui: &mut egui::Ui) {
    let controls_list = controls.get_controls().clone();

    for control in controls_list {
        match control {
            Control::Slider { name, min, max, .. } => {
                let mut value = controls.get_float(&name);
                if ui
                    .add(egui::Slider::new(&mut value, min..=max).text(&name))
                    .changed()
                {
                    controls.update_value(&name, ControlValue::Float(value));
                }
            }
            Control::Checkbox { name, .. } => {
                let mut value = controls.get_bool(&name);
                if ui.checkbox(&mut value, &name).changed() {
                    controls.update_value(&name, ControlValue::Bool(value));
                }
            }
            Control::Button { name } => if ui.button(&name).clicked() {},
            Control::Select { name, options, .. } => {
                let mut value = controls.get_string(&name);
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
                            }
                        }
                    });
            }
        }
    }
}