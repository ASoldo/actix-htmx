mod actors;
mod configs;
mod handlers;
mod models;
use crate::configs::config::MySanityConfig;
extern crate dotenv;
extern crate sanity;
use crate::handlers::handler::{
    about, content, cookie, events, get_comp, get_content, get_leaderboard, hello, index, login,
    logout, ws_index,
};
use crate::models::model::{Counter, TeraTemplates};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use dotenv::dotenv;
use futures::lock::Mutex;
use postgrest::Postgrest;
use tera::Tera;

use actix_web::{App, HttpServer};
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let tera_templates = Data::new(TeraTemplates {
        tera: Tera::new("templates/**/*").expect("Problem setting up Tera"),
    });

    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL not set");
    let supabase_public_key =
        std::env::var("SUPABASE_PUBLIC_KEY").expect("SUPABASE_PUBLIC_KEY not set");

    let supabase = Postgrest::new(supabase_url).insert_header("apikey", supabase_public_key);
    let sb = Data::new(supabase);

    let counter = Data::new(Counter { count: Mutex::new(0) });

    let sanity_token_key = std::env::var("SANITY_TOKEN_KEY").expect("SANITY_TOKEN_KEY not set");
    let sanity_project_id = std::env::var("SANITY_PROJECT_ID").expect("SANITY_PROJECT_ID not set");

    let sanity: MySanityConfig = MySanityConfig {
        sanity_config: Mutex::new(sanity::create(
            &sanity_project_id,
            "production",
            &sanity_token_key,
            true,
        )),
    };
    let sanity_config = Data::new(sanity);
    HttpServer::new(move || {
        App::new()
            .app_data(sanity_config.clone())
            .app_data(sb.clone())
            .app_data(counter.clone())
            .app_data(tera_templates.clone())
            .service(get_content)
            .service(login)
            .service(logout)
            .service(about)
            .service(content)
            .service(get_leaderboard)
            .service(index)
            .service(hello)
            .service(events)
            .service(ws_index)
            .service(cookie)
            .service(get_comp)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
