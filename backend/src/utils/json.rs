use axum::{
    async_trait,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct JsonExtractor<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for JsonExtractor<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                (StatusCode::BAD_REQUEST, rejection.to_string())
            })?;

        value.validate().map_err(|rejection| {
            let message = format!("Validation error: [{}]", rejection);
            (StatusCode::BAD_REQUEST, message)
        })?;

        Ok(Self(value))
    }
}
