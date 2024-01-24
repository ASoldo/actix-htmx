use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::Tera;

pub struct Counter {
    pub count: Mutex<i32>,
}

pub struct TeraTemplates {
    pub tera: Tera,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupabaseLoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub expires_at: i64,
    pub refresh_token: String,
    pub user: SupabaseUser,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupabaseUser {
    pub id: String,
    pub aud: String,
    pub role: String,
    pub email: String,
    pub email_confirmed_at: Option<String>,
    pub phone: String,
    pub confirmation_sent_at: Option<String>,
    pub confirmed_at: Option<String>,
    pub last_sign_in_at: Option<String>,
    pub app_metadata: HashMap<String, serde_json::Value>,
    pub user_metadata: HashMap<String, serde_json::Value>,
    pub identities: Vec<SupabaseIdentity>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupabaseIdentity {
    pub identity_id: String,
    pub id: String,
    pub user_id: String,
    pub identity_data: HashMap<String, String>,
    pub provider: String,
    pub last_sign_in_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "_type")]
    pub _type: String,
    pub question: String,
    #[serde(rename = "_createdAt")]
    pub _created_at: String,
    pub name: String,
    pub active: bool,
    pub description: String,
    pub _id: String,
    #[serde(rename = "_updatedAt")]
    pub _updated_at: String,
    pub slug: Slug,
    pub image: Image,
    // Add other fields as needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slug {
    pub current: String,
    pub _type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub _type: String,
    pub asset: Asset,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub _ref: String,
    pub _type: String,
}

#[derive(Serialize)]
pub struct Navigation {
    pub current_page: String,
}

impl Navigation {
    pub fn new(current_page: &str) -> Self {
        Navigation { current_page: current_page.to_string() }
    }
}

pub struct MySanityConfig {
    pub sanity_config: Mutex<sanity::SanityConfig>,
}
