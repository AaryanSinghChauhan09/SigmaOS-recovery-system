use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub size: u64,
    pub include_user_data: bool,
    pub include_packages: bool,
}
