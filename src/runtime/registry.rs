use nannou::prelude::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::str;
use std::sync::RwLock;

use crate::framework::prelude::*;

#[macro_export]
macro_rules! register_legacy_sketches {
  ($registry:expr, $($module:ident),*) => {
      $(
          $registry.register(
              &crate::sketches::$module::SKETCH_CONFIG,
              |app, rect| {
                  let model = crate::sketches::$module::init_model(
                      app,
                      WindowRect::new(rect)
                  );
                  Box::new(SketchAdapter::new(
                      model,
                      crate::sketches::$module::update,
                      crate::sketches::$module::view,
                  ))
              }
          );
      )*
  };
}

#[macro_export]
macro_rules! register_sketches {
  ($registry:expr, $($module:ident),*) => {
      $(
          $registry.register(
              &crate::sketches::$module::SKETCH_CONFIG,
              |app, rect| {
                  Box::new(crate::sketches::$module::init(
                      app,
                      WindowRect::new(rect)
                  ))
              }
          );
      )*
  };
}

pub struct SketchInfo {
    pub config: &'static SketchConfig,
    pub factory: Box<
        dyn for<'a> Fn(&'a App, Rect) -> Box<dyn Sketch + 'static>
            + Send
            + Sync,
    >,
}

pub static REGISTRY: Lazy<RwLock<SketchRegistry>> =
    Lazy::new(|| RwLock::new(SketchRegistry::new()));

pub struct SketchRegistry {
    sketches: HashMap<String, SketchInfo>,
    sorted_names: Option<Vec<String>>,
}

impl SketchRegistry {
    fn new() -> Self {
        Self {
            sketches: HashMap::new(),
            sorted_names: None,
        }
    }

    pub fn register<F>(&mut self, config: &'static SketchConfig, factory: F)
    where
        F: Fn(&App, Rect) -> Box<dyn Sketch> + Send + Sync + 'static,
    {
        self.sketches.insert(
            config.name.to_string(),
            SketchInfo {
                config,
                factory: Box::new(factory),
            },
        );
        self.sorted_names = None;
    }

    pub fn get(&self, name: &str) -> Option<&SketchInfo> {
        self.sketches.get(name)
    }

    pub fn prepare(&mut self) {
        if self.sorted_names.is_none() {
            let mut names: Vec<String> =
                self.sketches.keys().cloned().collect();
            names.sort();
            self.sorted_names = Some(names);
        }
    }

    pub fn names(&self) -> &Vec<String> {
        self.sorted_names.as_ref().expect(
            "Registry must be prepared before accessing names. \
                Call prepare() first.",
        )
    }
}
