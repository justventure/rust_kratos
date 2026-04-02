use reqwest::{Client, StatusCode, header};

use crate::domain::value_objects::flow_id::FlowId;
use crate::infrastructure::adapters::kratos::models::errors::KratosFlowError;
use crate::infrastructure::adapters::kratos::models::flows::{FlowResult, PostFlowResult};

pub async fn fetch_flow(
    client: &Client,
    public_url: &str,
    endpoint: &str,
    cookie: Option<&str>,
) -> Result<FlowResult, KratosFlowError> {
    let url = format!("{}/self-service/{}/browser", public_url, endpoint).replace("localhost", "127.0.0.1");

    let mut request = client.get(&url);
    if let Some(cookie_value) = cookie {
        request = request.header(header::COOKIE, cookie_value);
    }

    let response = request.send().await.map_err(KratosFlowError::network)?;

    let status = response.status();
    let flow_cookies = extract_cookies(&response);

    if status == StatusCode::SEE_OTHER || status == StatusCode::FOUND {
        return handle_redirect(client, public_url, endpoint, response, flow_cookies, cookie).await;
    }

    if !status.is_success() {
        let body = response
            .json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));
        return Err(KratosFlowError { status, body });
    }

    let flow: serde_json::Value = response.json().await.map_err(KratosFlowError::network)?;

    let csrf_token = extract_csrf_token(&flow).map_err(KratosFlowError::network)?;
    let flow_id = extract_flow_id(&flow).map_err(KratosFlowError::network)?;
    let mut all_cookies = cookie.map(|c| vec![c.to_string()]).unwrap_or_default();
    all_cookies.extend(flow_cookies);

    Ok(FlowResult {
        flow_id,
        csrf_token,
        cookies: all_cookies,
    })
}

pub async fn post_flow(
    client: &Client,
    public_url: &str,
    endpoint: &str,
    flow_id: &FlowId,
    data: serde_json::Value,
    cookies: &[String],
) -> Result<PostFlowResult, KratosFlowError> {
    let url =
        format!("{}/self-service/{}?flow={}", public_url, endpoint, flow_id.as_str()).replace("localhost", "127.0.0.1");

    let response = client
        .post(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, cookies.join("; "))
        .json(&data)
        .send()
        .await
        .map_err(KratosFlowError::network)?;

    let response_cookies = extract_cookies(&response);
    let status = response.status();

    if status == StatusCode::SEE_OTHER || status == StatusCode::FOUND {
        let body = response
            .json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));
        return Err(KratosFlowError { status, body });
    }

    if !status.is_success() {
        let body = response
            .json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));
        return Err(KratosFlowError { status, body });
    }

    let data = response
        .json::<serde_json::Value>()
        .await
        .map_err(KratosFlowError::network)?;

    Ok(PostFlowResult {
        data,
        cookies: response_cookies,
    })
}

fn extract_cookies(response: &reqwest::Response) -> Vec<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .collect()
}

fn extract_csrf_token(flow: &serde_json::Value) -> Result<String, String> {
    flow["ui"]["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find(|node| node["attributes"]["name"].as_str() == Some("csrf_token"))
        })
        .and_then(|node| node["attributes"]["value"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "CSRF token not found in flow response".to_string())
}

fn extract_flow_id(flow: &serde_json::Value) -> Result<FlowId, String> {
    flow["id"]
        .as_str()
        .map(FlowId::new)
        .ok_or_else(|| "Flow ID not found in response".to_string())
}

async fn handle_redirect(
    client: &Client,
    public_url: &str,
    endpoint: &str,
    response: reqwest::Response,
    flow_cookies: Vec<String>,
    cookie: Option<&str>,
) -> Result<FlowResult, KratosFlowError> {
    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| KratosFlowError::network("No redirect location found"))?;

    if !location.contains("flow=") {
        return Err(KratosFlowError {
            status: StatusCode::UNAUTHORIZED,
            body: serde_json::json!({ "error": "session required" }),
        });
    }

    let flow_id_str = location
        .split("flow=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .ok_or_else(|| KratosFlowError::network(format!("Flow ID not found in redirect URL: {}", location)))?;

    let flow_url = format!(
        "{}/self-service/{}/flows?id={}",
        public_url.replace("localhost", "127.0.0.1"),
        endpoint,
        flow_id_str
    );

    let mut all_cookies: Vec<String> = cookie.map(|c| vec![c.to_string()]).unwrap_or_default();
    all_cookies.extend(flow_cookies);

    let flow_response = client
        .get(&flow_url)
        .header(header::COOKIE, all_cookies.join("; "))
        .send()
        .await
        .map_err(KratosFlowError::network)?;

    let status = flow_response.status();

    if !status.is_success() {
        let body = flow_response
            .json::<serde_json::Value>()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));
        return Err(KratosFlowError { status, body });
    }

    let flow: serde_json::Value = flow_response.json().await.map_err(KratosFlowError::network)?;

    let csrf_token = extract_csrf_token(&flow).map_err(KratosFlowError::network)?;
    let flow_id = extract_flow_id(&flow).map_err(KratosFlowError::network)?;

    Ok(FlowResult {
        flow_id,
        csrf_token,
        cookies: all_cookies,
    })
}
