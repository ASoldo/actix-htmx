/// A module defining various models and structures used in the application.
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::Tera;

/// Counter structure that holds an atomic integer.
///
/// This structure is typically used for keeping track of a count value
/// in a thread-safe manner using asynchronous mutex.
pub struct Counter {
    pub count: Mutex<i32>,
}

/// TeraTemplates structure that encapsulates Tera template engine.
///
/// This structure holds an instance of `Tera` which can be used to render
/// templates. It is typically used in the context of web response rendering.
pub struct TeraTemplates {
    pub tera: Tera,
}

/// A struct representing a login request with email and password fields.
///
/// This structure is used to deserialize login request data from clients.
#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// A struct representing a user.
///
/// This includes user's basic information such as ID, name, and email.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

/// A struct representing the response from Supabase upon successful login.
///
/// It includes the access token, token type, expiry information, and user details.
#[derive(Serialize, Deserialize, Debug)]
pub struct SupabaseLoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub expires_at: i64,
    pub refresh_token: String,
    pub user: SupabaseUser,
}

/// A struct representing a user as returned by the Supabase authentication API.
///
/// It includes detailed user information such as ID, email, roles, and metadata.
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

/// A struct representing an identity associated with a Supabase user.
///
/// This includes identity-specific details such as the provider and timestamps.
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

/// A struct representing an item, typically fetched from a database or API.
///
/// This includes information about the item such as type, name, description, and related media.
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
}

/// A struct representing a slug, typically used in URLs or as identifiers.
#[derive(Debug, Serialize, Deserialize)]
pub struct Slug {
    pub current: String,
    pub _type: String,
}

/// A struct representing an image, usually associated with an item.
///
/// It includes information about the image type and its asset.
#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub _type: String,
    pub asset: Asset,
}

/// A struct representing an asset, which could be an image or other media type.
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub _ref: String,
    pub _type: String,
}

/// A struct for maintaining navigation context, particularly for UI rendering.
///
/// It includes the current page information.
#[derive(Serialize)]
pub struct Navigation {
    pub current_page: String,
}

impl Navigation {
    /// Creates a new Navigation instance with the specified current page.
    pub fn new(current_page: &str) -> Self {
        Navigation { current_page: current_page.to_string() }
    }
}

/// A struct wrapping Sanity configuration for use in application contexts.
///
/// It contains a thread-safe, asynchronously-locked `SanityConfig`.
pub struct MySanityConfig {
    pub sanity_config: Mutex<sanity::SanityConfig>,
}
