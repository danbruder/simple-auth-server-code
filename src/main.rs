// main.rs
#![allow(dead_code)] // usful in dev mode
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
use crate::models::DbExecutor;
use actix::prelude::*;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use chrono::Duration;
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;

mod auth_handler;
mod auth_routes;
mod email_service;
mod errors;
mod invitation_handler;
mod invitation_routes;
mod models;
mod register_handler;
mod register_routes;
mod schema;
mod utils;

fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var(
        "RUST_LOG",
        "simple-auth-server=debug,actix_web=info,actix_server=info",
    );

    env_logger::init();

    let sys = actix_rt::System::new("example");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let domain = std::env::var("DOMAIN").expect("DOMAIN must be set");
    let secret = std::env::var("SECRET").expect("SECRET must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let address: Addr<DbExecutor> = SyncArbiter::start(4, move || DbExecutor(pool.clone()));

    HttpServer::new(move || {
        App::new()
            .data(address.clone())
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(secret.as_bytes())
                    .name("auth")
                    .path("/")
                    .domain(domain.as_str())
                    .max_age_time(Duration::days(30))
                    .secure(false),
            ))
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/auth")
                            .route(web::post().to_async(auth_routes::login))
                            .route(web::delete().to(auth_routes::logout))
                            .route(web::get().to_async(auth_routes::get_me)),
                    )
                    .service(
                        web::resource("/invitation")
                            .route(web::post().to_async(invitation_routes::register_email)),
                    )
                    .service(
                        web::resource("/register/{invitation_id}")
                            .route(web::post().to_async(register_routes::register_user)),
                    ),
            )
    })
    .bind("127.0.0.1:3000")?
    .start();
    sys.run()
}
