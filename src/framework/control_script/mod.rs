//! Provides a means of controlling sketch parameters with the various Lattice
//! control systems from an external yaml file that can be hot-reloaded.
//!
#![doc = include_str!("../../../docs/control_script_reference.md")]

pub mod control_script;
pub use control_script::ControlScript;

mod config;

mod control_script_builder;
pub use control_script_builder::ControlScriptBuilder;

mod dep_graph;
mod eval_cache;
mod param_mod;
