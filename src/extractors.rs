use axum::{
    async_trait,
    extract::{FromRequest, Request, Multipart},
    http::StatusCode,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, Map, Number};

pub struct FormOrJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for FormOrJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if content_type.starts_with("multipart/form-data") {
            let mut multipart = Multipart::from_request(req, state).await.map_err(|e| {
                (StatusCode::BAD_REQUEST, e.to_string())
            })?;
            
            let mut map = Map::new();
            while let Ok(Some(field)) = multipart.next_field().await {
                let name = field.name().unwrap_or("").to_string();
                if name.is_empty() { continue; }
                
                if let Ok(text) = field.text().await {
                    if let Ok(json_val) = serde_json::from_str(&text) {
                        map.insert(name, json_val);
                    } else if let Ok(num) = text.parse::<i64>() {
                        map.insert(name, Value::Number(Number::from(num)));
                    } else if let Ok(num) = text.parse::<f64>() {
                        if let Some(n) = Number::from_f64(num) {
                            map.insert(name, Value::Number(n));
                        } else {
                            map.insert(name, Value::String(text));
                        }
                    } else if text == "true" {
                        map.insert(name, Value::Bool(true));
                    } else if text == "false" {
                        map.insert(name, Value::Bool(false));
                    } else if text == "null" || text.is_empty() {
                        map.insert(name, Value::Null);
                    } else {
                        map.insert(name, Value::String(text));
                    }
                }
            }
            
            let value = Value::Object(map);
            match serde_json::from_value(value) {
                Ok(parsed) => Ok(FormOrJson(parsed)),
                Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
            }
        } else if content_type.starts_with("application/json") {
            let axum::extract::Json(payload) = axum::extract::Json::<T>::from_request(req, state).await.map_err(|e| {
                (StatusCode::BAD_REQUEST, e.to_string())
            })?;
            Ok(FormOrJson(payload))
        } else {
            Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Content type must be application/json or multipart/form-data".into(),
            ))
        }
    }
}
