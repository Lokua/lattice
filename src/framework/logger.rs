use env_logger::{Builder, Env};
use log::LevelFilter;

pub fn init_logger() {
    Builder::from_env(Env::default().default_filter_or("lattice=info"))
        .filter_module("nannou", LevelFilter::Warn)
        .init();
}
