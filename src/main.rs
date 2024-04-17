mod validation;

use base64::{engine::general_purpose::{self}, Engine};
use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestExt, Response};
use serde_json::json;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let params: Result<std::collections::HashMap<String, String>, (u16, String)> =
        validation::validate_request(event.path_parameters_ref(), event.headers());

    if params.is_err() {
        let (status_code, message) = params.unwrap_err();
        return Ok(Response::builder()
            .status(status_code)
            .header("content-type", "text/plain")
            .body(message.into())
            .map_err(Box::new)?);
    }

    let params = params.unwrap();

    let event_body = general_purpose::STANDARD.encode(event.body().as_ref());
    let event_body = event_body.as_str();

    let body = json!({
        "message": {
            "token": params.get("deviceToken").unwrap(),
            "data": {
                "p": event_body,
                "k": params.get("publicKey").unwrap(),
                "s": params.get("salt").unwrap(),
                "x": params.get("pushAccountId").unwrap(),
            }
        }
    });

    let project_id = std::env::var("PROJECT_ID");

    if project_id.is_err() {
        return Ok(Response::builder()
            .status(500)
            .header("content-type", "text/plain")
            .body("Internal Server Error: PROJECT_ID not set".into())
            .map_err(Box::new)?);
    }

    let project_id = project_id.unwrap();

    let fcm_res = reqwest::Client::new().post(format!("https://fcm.googleapis.com/v1/projects/{project_id}/messages:send")).json(&body).send().await?;

    match fcm_res.error_for_status() {
        Ok(_res) => {
            return Ok(Response::builder().status(204).body(Body::Empty)?);
        }
        Err(_err) => {
            return Ok(Response::builder()
                .status(500)
                .header("content-type", "text/plain")
                .body("Unknown error when sending FCM message".into())
                .map_err(Box::new)?)
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
