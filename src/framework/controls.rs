use nannou_egui::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ControlValue {
    Float(f32),
    Bool(bool),
    String(String),
}

type DisabledFn = Option<Box<dyn Fn(&Controls) -> bool>>;

#[derive(Serialize, Deserialize)]
pub enum Control {
    Slider {
        name: String,
        value: f32,
        min: f32,
        max: f32,
        step: f32,
        #[serde(skip)]
        disabled: DisabledFn,
    },
    Checkbox {
        name: String,
        value: bool,
        #[serde(skip)]
        disabled: DisabledFn,
    },
    Select {
        name: String,
        value: String,
        options: Vec<String>,
        #[serde(skip)]
        disabled: DisabledFn,
    },
    Button {
        name: String,
        #[serde(skip)]
        disabled: DisabledFn,
    },
    Separator {},
}

impl Control {
    pub fn name(&self) -> &str {
        match self {
            Control::Slider { name, .. } => name,
            Control::Checkbox { name, .. } => name,
            Control::Select { name, .. } => name,
            Control::Button { name, .. } => name,
            Control::Separator {} => "",
        }
    }

    pub fn value(&self) -> ControlValue {
        match self {
            Control::Slider { value, .. } => ControlValue::Float(*value),
            Control::Checkbox { value, .. } => ControlValue::Bool(*value),
            Control::Select { value, .. } => {
                ControlValue::String(value.clone())
            }
            Control::Button { .. } => ControlValue::Bool(false),
            Control::Separator { .. } => ControlValue::Bool(false),
        }
    }

    pub fn checkbox(name: &str, value: bool) -> Control {
        Control::Checkbox {
            name: name.to_string(),
            value,
            disabled: None,
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
            disabled: None,
        }
    }

    pub fn select(name: &str, value: &str, options: Vec<String>) -> Control {
        Control::Select {
            name: name.into(),
            value: value.into(),
            options,
            disabled: None,
        }
    }

    pub fn slider_x<F>(
        name: &str,
        value: f32,
        range: (f32, f32),
        step: f32,
        disabled: F,
    ) -> Control
    where
        F: Fn(&Controls) -> bool + 'static,
    {
        Control::Slider {
            name: name.to_string(),
            value,
            min: range.0,
            max: range.1,
            step,
            disabled: Some(Box::new(disabled)),
        }
    }

    pub fn checkbox_x<F>(name: &str, value: bool, disabled: F) -> Control
    where
        F: Fn(&Controls) -> bool + 'static,
    {
        Control::Checkbox {
            name: name.to_string(),
            value,
            disabled: Some(Box::new(disabled)),
        }
    }

    pub fn separator() -> Control {
        Control::Separator {}
    }

    pub fn select_x<F>(
        name: &str,
        value: &str,
        options: Vec<String>,
        disabled: F,
    ) -> Control
    where
        F: Fn(&Controls) -> bool + 'static,
    {
        Control::Select {
            name: name.into(),
            value: value.into(),
            options,
            disabled: Some(Box::new(disabled)),
        }
    }

    fn is_disabled(&self, controls: &Controls) -> bool {
        match self {
            Control::Slider { disabled, .. }
            | Control::Button { disabled, .. }
            | Control::Checkbox { disabled, .. }
            | Control::Select { disabled, .. } => {
                disabled.as_ref().map_or(false, |f| f(controls))
            }
            _ => false,
        }
    }
}

pub type ControlValues = HashMap<String, ControlValue>;

#[derive(Serialize, Deserialize)]
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

    /// Retrieves the original control configuration by name
    ///
    /// # Arguments
    /// * `name` - The name of the control to retrieve
    ///
    /// # Returns
    /// An Option containing a reference to the Control if found, None otherwise
    pub fn get_original_config(&self, name: &str) -> Option<&Control> {
        self.controls.iter().find(|control| control.name() == name)
    }

    /// Helper method to get min and max values for a slider control
    ///
    /// # Arguments
    /// * `name` - The name of the slider control
    ///
    /// # Returns
    /// A tuple of (min, max) values if the control has one, otherwise panics.
    pub fn slider_range(&self, name: &str) -> (f32, f32) {
        self.get_original_config(name)
            .and_then(|control| match control {
                Control::Slider { min, max, .. } => Some((*min, *max)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("Unable to find range for {}", name))
    }
}

pub fn draw_controls(controls: &mut Controls, ui: &mut egui::Ui) -> bool {
    let mut any_changed = false;
    let mut updates = Vec::new();

    for control in &controls.controls {
        let is_disabled = control.is_disabled(controls);

        match control {
            Control::Slider {
                name,
                min,
                max,
                step,
                ..
            } => {
                let mut value = controls.float(name);
                if ui
                    .add_enabled(
                        !is_disabled,
                        egui::Slider::new(&mut value, *min..=*max)
                            .text(name)
                            .step_by((*step).into()),
                    )
                    .changed()
                {
                    updates.push((name.clone(), ControlValue::Float(value)));
                    any_changed = true;
                }
            }
            Control::Checkbox { name, .. } => {
                let mut value = controls.bool(name);
                if ui
                    .add_enabled(
                        !is_disabled,
                        egui::Checkbox::new(&mut value, name),
                    )
                    .changed()
                {
                    updates.push((name.clone(), ControlValue::Bool(value)));
                    any_changed = true;
                }
            }
            Control::Select { name, options, .. } => {
                let mut value = controls.string(name);
                let name_clone = name.clone();
                egui::ComboBox::from_label(name)
                    .selected_text(&value)
                    .show_ui(ui, |ui| {
                        ui.set_enabled(!is_disabled);
                        for option in options {
                            if ui
                                .selectable_value(
                                    &mut value,
                                    option.clone(),
                                    option,
                                )
                                .changed()
                            {
                                updates.push((
                                    name_clone.clone(),
                                    ControlValue::String(value.clone()),
                                ));
                                any_changed = true;
                            }
                        }
                    });
            }
            Control::Button { name, .. } => {
                if ui
                    .add_enabled(!is_disabled, egui::Button::new(name))
                    .clicked()
                {
                    // Handle click
                }
            }
            Control::Separator {} => {
                ui.separator();
            }
        }
    }

    for (name, value) in updates {
        controls.update_value(&name, value);
    }

    any_changed
}
