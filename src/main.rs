use fastly::http::{header, Method, StatusCode};
use fastly::{ConfigStore, Request, Response, SecretStore};
use serde::{Deserialize, Serialize};

const UPDATE_URL: &str = "https://api.porkbun.com/api/json/v3/dns/editByNameType";

enum Error {
    StatusCode(StatusCode),
}

#[derive(Serialize)]
struct UpdateRequest {
    #[serde(rename = "apikey")]
    api_key: String,
    #[serde(rename = "secretapikey")]
    secret_key: String,
    content: String,
}

#[derive(Deserialize)]
struct UpdateResponse {
    status: String,
    message: Option<String>,
}

#[fastly::main]
fn main(req: Request) -> Result<Response, fastly::Error> {
    match handle_request(req) {
        Ok(resp) => Ok(resp),
        Err(Error::StatusCode(status)) => Ok(Response::from_status(status)),
    }
}

fn handle_request(req: Request) -> Result<Response, Error> {
    println!(
        "Incoming request: version={} method={} url={}",
        std::env::var("FASTLY_SERVICE_VERSION").unwrap(),
        req.get_method(),
        req.get_url(),
    );

    if req.get_method() != Method::GET {
        return Err(Error::StatusCode(StatusCode::METHOD_NOT_ALLOWED));
    }

    if req.get_path() != "/nic/update" {
        return Err(Error::StatusCode(StatusCode::NOT_FOUND));
    }

    let hostname = req
        .get_query_parameter("hostname")
        .ok_or(Error::StatusCode(StatusCode::BAD_REQUEST))?;
    let ip = req
        .get_query_parameter("myip")
        .ok_or(Error::StatusCode(StatusCode::BAD_REQUEST))?;
    let auth_token = req
        .get_query_parameter("auth_token")
        .ok_or(Error::StatusCode(StatusCode::UNAUTHORIZED))?;

    println!("Updating {} to {}", hostname, ip);

    let config_store = ConfigStore::open("porkbun_config");
    let domain = config_store.get("domain").unwrap();

    let secret_store = SecretStore::open("porkbun_secrets").unwrap();
    let api_key =
        String::from_utf8(secret_store.get("api_key").unwrap().plaintext().to_vec()).unwrap();
    let secret_key =
        String::from_utf8(secret_store.get("secret_key").unwrap().plaintext().to_vec()).unwrap();
    let expected_auth_token =
        String::from_utf8(secret_store.get("auth_token").unwrap().plaintext().to_vec()).unwrap();

    if auth_token != expected_auth_token {
        return Err(Error::StatusCode(StatusCode::UNAUTHORIZED));
    }

    let hostname = hostname
        .strip_suffix(&format!(".{}", domain))
        .unwrap_or(hostname);

    let update_request = UpdateRequest {
        api_key,
        secret_key,
        content: ip.to_string(),
    };

    let url = format!("{}/{}/A/{}", UPDATE_URL, domain, hostname);
    let mut beresp = Request::post(url)
        .with_pass(true)
        .with_header(header::CONTENT_TYPE, "application/json")
        .with_body(serde_json::to_string(&update_request).unwrap())
        .send("porkbun")
        .map_err(|_| Error::StatusCode(StatusCode::SERVICE_UNAVAILABLE))?;

    let update_response: UpdateResponse = serde_json::from_reader(beresp.take_body()).unwrap();
    if update_response.status != "SUCCESS" {
        println!(
            "Failed to update: {}",
            update_response.message.unwrap_or_default()
        );
        return Err(Error::StatusCode(StatusCode::BAD_REQUEST));
    }

    Ok(Response::from_status(StatusCode::OK))
}
