use platano::{
    config::get_config,
    llm::types::{Message, Role},
    startup::build,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("platano".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_config().expect("Failed to read config");
    let app = build(settings)?;

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Greet me.".into());

    let response = app
        .llm
        .chat(vec![Message {
            role: Role::User,
            content: prompt,
        }])
        .await?;

    println!("{}", response.message.content);

    Ok(())
}
