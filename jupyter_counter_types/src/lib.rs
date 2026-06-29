#[derive(serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum CounterBackendMessage {
    Increment,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum CounterFrontendMessage {
    NewValue { value: u64 },
}
