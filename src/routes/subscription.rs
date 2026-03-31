use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, UserEmail, UserName};

#[derive(serde::Deserialize)]
pub struct SubscriptionFormData {
    email: String,
    name: String,
}

impl TryFrom<SubscriptionFormData> for NewSubscriber {
    type Error = String;
    fn try_from(value: SubscriptionFormData) -> Result<Self, Self::Error> {
        let name = UserName::parse(value.name)?;
        let email = UserEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<SubscriptionFormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let subscriber = match form.0.try_into() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to create new subscriber: {:?}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match insert_subscriber(&pool, &subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscription (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early
        // if the function failed, returning sqlx::Error
        // Will fix this later
    })?;
    Ok(())
}
