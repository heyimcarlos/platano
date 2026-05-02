use platano::{
    config::{Environment, get_config},
    startup::build,
    telemetry::{get_pretty_subscriber, get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let environment = Environment::from_env();

    let subscriber = match environment {
        Environment::Production => get_subscriber("platano".into(), "info".into(), std::io::stdout),
        Environment::Local => get_pretty_subscriber("info,platano=debug".into()),
    };
    init_subscriber(subscriber);

    let settings = get_config().expect("Failed to read config");
    let agent = build(settings)?;

    let prompt = std::env::args().nth(1).unwrap_or_else(|| "hello".into());
    let response = agent.run(prompt).await?;
    println!("Agent response: {}", response);

    Ok(())
}
