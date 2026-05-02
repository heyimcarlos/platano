use platano::{
    config::{LlMSettings, Settings},
    startup::build,
};
use wiremock::{Mock, MockServer};

struct TestApp {
    pub agent: Agent,
    pub llm_mock: MockServer,
}

async fn spawn_app() -> TestApp {
    let llm_mock = MockServer::start().await;

    let settings = Settings {
        llm: LlMSettings {
            base_url: llm_mock.uri(),
            model: "test".into(),
            temperature: None,
        },
    };

    let app = build(settings).expect("Failed to build app");
    TestApp {
        agent: app.agent,
        llm_mock,
    }
}

// #[tokio::test]
// async fn agent_executes_tool_and_returns_final_answer() {
//     let test_app = spawn_app().await;
// }
