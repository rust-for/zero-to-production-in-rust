use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;

use crate::routes::{health_check, subscribe};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connection in an Arc smart pointer
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added using the `wrap` method on `App`
            .wrap(TracingLogger)
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Register the connection as part of the application state
            // Our pool is already wrapped in an Arc pointer:
            // using .data would add another Arc pointer on top of
            // the existing one - an unnecessary indirection.
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
