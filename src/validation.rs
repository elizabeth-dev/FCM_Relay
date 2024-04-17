use std::{collections::HashMap, str::FromStr};

use lambda_http::{aws_lambda_events::query_map::QueryMap, http};
use reqwest::header::{self, HeaderName};

pub(crate) fn validate_request(
    query: Option<&QueryMap>,
    headers: &http::HeaderMap<http::HeaderValue>,
) -> Result<HashMap<String, String>, (u16, String)> {
    let mut map: HashMap<String, String> = HashMap::new();

    if query.is_none() {
        return Result::Err((400, "Bad Request: no URL parameters".to_string()));
    }

    let query = query.unwrap();

    let device_token = &query.first("deviceToken");
    if device_token.is_none() {
        return Result::Err((400, "Bad Request: deviceToken missing".to_string()));
    }

    map.insert("deviceToken".to_string(), device_token.unwrap().to_string());

    let push_account_id = &query.first("pushAccountId");
    if push_account_id.is_none() {
        return Result::Err((400, "Bad Request: pushAccountId missing".to_string()));
    }

    map.insert(
        "pushAccountId".to_string(),
        push_account_id.unwrap().to_string(),
    );

    if !headers
        .get(header::CONTENT_ENCODING)
        .is_some_and(|x| x == "aesgcm")
    {
        return Result::Err((
            400,
            "Bad Request: Content-Encoding must be aesgcm".to_string(),
        ));
    }

    let public_key = get_header_param(headers, HeaderName::from_str("Crypto-Key").unwrap(), "dh")
        .unwrap_or_default();

    if public_key.is_empty() {
        return Result::Err((400, "Bad Request: dh missing in Crypto-Key".to_string()));
    }

    map.insert("publicKey".to_string(), public_key);

    let salt = get_header_param(headers, HeaderName::from_str("Encryption").unwrap(), "salt")
        .unwrap_or_default();

    if salt.is_empty() {
        return Result::Err((400, "Bad Request: salt missing in Encryption".to_string()));
    }

    map.insert("salt".to_string(), salt);

    Ok(map)
}

fn get_header_param(headers: &http::HeaderMap, key: HeaderName, param: &str) -> Option<String> {
    return headers
        .get(key)
        .unwrap()
        .to_str()
        .unwrap_or_default()
        .split(';')
        .find(|x| x.starts_with(param))
        .and_then(|x| x.split('=').last().map(|x| x.to_string()));
}
