use platano::{
    config::get_config,
    telemetry::{get_subscriber, init_subscriber},
};

fn main() {
    let subscriber = get_subscriber("platano".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to read config");
    println!("Config: {:#?}", config);
}
