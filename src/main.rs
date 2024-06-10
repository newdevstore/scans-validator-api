use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use reqwest::Error;
use serde::Serialize;
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::env;
use std::str::FromStr;

#[derive(Serialize)]
struct ApiResponse {
    status_code: u16,
    message: String,
    data: Option<Value>,
}

#[get("/ethereum/{tx_hash}")]
async fn get_ethereum(path: web::Path<String>) -> impl Responder {
    let etherscan_api_key: String =
        env::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY not set in environment");
    let tx_hash = path.into_inner();
    match get_ethereum_transaction(&tx_hash, &etherscan_api_key).await {
        Ok(data) => HttpResponse::Ok().json(ApiResponse {
            status_code: 200,
            message: "Ethereum Transaction found".to_string(),
            data: Some(data),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            status_code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

#[get("/polygon/{tx_hash}")]
async fn get_polygon(path: web::Path<String>) -> impl Responder {
    let polygonscan_api_key =
        env::var("POLYGONSCAN_API_KEY").expect("POLYGONSCAN_API_KEY not set in environment");
    let tx_hash = path.into_inner();
    match get_polygon_transaction(&tx_hash, &polygonscan_api_key).await {
        Ok(data) => HttpResponse::Ok().json(ApiResponse {
            status_code: 200,
            message: "Polygon Transaction found".to_string(),
            data: Some(data),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            status_code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

#[get("/bsc/{tx_hash}")]
async fn get_bsc(path: web::Path<String>) -> impl Responder {
    let bscscan_api_key = env::var("BSCSCAN_API_KEY").expect("BSCSCAN_API_KEY not set in environment");
    let tx_hash = path.into_inner();
    match get_bsc_transaction(&tx_hash, &bscscan_api_key).await {
        Ok(data) => HttpResponse::Ok().json(ApiResponse {
            status_code: 200,
            message: "BSC Transaction found".to_string(),
            data: Some(data),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            status_code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

#[get("/solana/{tx_hash}")]
async fn get_solana(path: web::Path<String>) -> impl Responder {
    let tx_hash = path.into_inner();
    match get_solana_transaction(&tx_hash).await {
        Ok(data) => HttpResponse::Ok().json(ApiResponse {
            status_code: 200,
            message: "Solana Transaction found".to_string(),
            data: Some(data),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
            status_code: 500,
            message: e.to_string(),
            data: None,
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    HttpServer::new(|| {
        App::new()
            .service(get_ethereum)
            .service(get_polygon)
            .service(get_bsc)
            .service(get_solana)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn get_ethereum_transaction(tx_hash: &str, api_key: &str) -> Result<Value, Error> {
    let url = format!(
        "https://api.etherscan.io/api?module=proxy&action=eth_getTransactionByHash&txhash={}&apikey={}",
        tx_hash, api_key
    );
    let resp = reqwest::get(&url).await?.json::<Value>().await?;
    Ok(resp)
}

async fn get_polygon_transaction(tx_hash: &str, api_key: &str) -> Result<Value, Error> {
    let url = format!(
        "https://api.polygonscan.com/api?module=proxy&action=eth_getTransactionByHash&txhash={}&apikey={}",
        tx_hash, api_key
    );
    let resp = reqwest::get(&url).await?.json::<Value>().await?;
    Ok(resp)
}

async fn get_bsc_transaction(tx_hash: &str, api_key: &str) -> Result<Value, Error> {
    let url = format!(
        "https://api.bscscan.com/api?module=proxy&action=eth_getTransactionByHash&txhash={}&apikey={}",
        tx_hash, api_key
    );
    let resp = reqwest::get(&url).await?.json::<Value>().await?;
    Ok(resp)
}

async fn get_solana_transaction(tx_hash: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    let signature = Signature::from_str(tx_hash)?;
    let transaction = tokio::task::spawn_blocking(move || {
        client.get_transaction(&signature, UiTransactionEncoding::Json)
    })
    .await??;
    let json_transaction = serde_json::to_value(transaction)?;
    Ok(json_transaction)
}

#[get("/solana-balances")]
async fn get_solana_balances(
    rpc: web::Query<String>,
    public_keys: web::Query<Vec<String>>,
) -> HttpResponse {
    let rpc_url = rpc.into_inner();
    let public_keys = public_keys.into_inner();

    let mut accounts_with_balance = Vec::new();

    let client = RpcClient::new(rpc_url);

    for public_key_str in public_keys {
        if let Ok(public_key) = Pubkey::from_str(&public_key_str) {
            if let Ok(balance) = client.get_balance(&public_key) {
                if balance > 0 {
                    let sol_balance = balance as f64 / 1e9;
                    accounts_with_balance
                        .push(format!("{}: {} SOL", public_key_str, sol_balance));
                }
            }
        }
    }

    HttpResponse::Ok().json(accounts_with_balance)
}
