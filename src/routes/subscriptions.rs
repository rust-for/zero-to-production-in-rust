use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    let request_id = Uuid::new_v4();

    // Spans, like logs, have an associated level
    // `info_span` creates a span at the info-level
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        email = %form.email,
        name = %form.name
    );

    // Using `enter` in an async function is a recipe for disaster!
    // Bear with me for now, but don't do this at home.
    // See the following section on `tracing-futures`
    let _request_span_guard = request_span.enter();

    // We are using the same interpolation syntax of `println`/`print` here!
    // tracing::info!("request_id {} - Adding '{}' '{}' as a new subscriber.", request_id, form.email, form.name);
    // tracing::info!("request_id {} - Saving new subscriber details in the database", request_id);

    // We do not call `.enter` on query_span!
    // `.instrument` takes care of it at the right moments
    // in the query future lifetime
    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    // There is a bit of cerenomy here to get our hands on a &PgConnection.
    // web::Data<Arc<PgConnection>> is equivalent to Arc<Arc<PgConnection>>
    // Therefore connection.get_ref() returns a &Arc<PgConnection>
    // which we can then deref to a &PgConnection.
    // We could have avoided the double Arc wrapping using .app_data()
    // instead of .data() in src/startup.rs
    .execute(pool.as_ref())
    .instrument(query_span)
    .await
    .map_err(|e| {
        tracing::error!(
            "request_id {} - Failed to execute query: {:?}",
            request_id,
            e
        );

        HttpResponse::InternalServerError().finish()
    })?;

    tracing::info!(
        "request_id {} - New subscriber details have been saved",
        request_id
    );

    Ok(HttpResponse::Ok().finish())
}
