use env_logger::Env;
use log::info;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    info!("Starting");
}