use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum ServerMessage {
    Error { detail: String },
    JobUpdate { job: JobUpdateData },
    MeshGenerated { mesh: serde_json::Value, skeleton: serde_json::Value, clip: serde_json::Value },
    MotionGenerated { clip: serde_json::Value },
    ChatReply { reply: String, entities: Vec<EntityData>, actions: Vec<ActionResult> },
    Progress { action: String, progress: f32, message: String },
}

#[derive(Clone, Deserialize, Debug)]
pub struct JobUpdateData {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub progress: f64,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRequest {
    pub job_type: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub job_request: Option<JobRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessageData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessageData {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChatResponse {
    pub reply: String,
    pub actions: Vec<ActionResult>,
    pub entities: Vec<EntityData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActionResult {
    pub action_type: String,
    pub status: String,
    pub entity_id: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EntityData {
    pub entity_type: String,
    pub label: String,
    #[serde(default)]
    pub entity_id: Option<u64>,
    pub data: serde_json::Value,
}
