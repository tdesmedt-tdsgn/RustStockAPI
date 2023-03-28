use actix_web::{get, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use ta::indicators::{ExponentialMovingAverage, RelativeStrengthIndex, SimpleMovingAverage};
use ta::Next;
use chrono::{Duration, Utc, NaiveDateTime};

const POLYGON_API_URL: &str = "https://api.polygon.io/v2/aggs/ticker";
const POLYGON_API_KEY: &str = "QK95hnin0k8IqtmpTZfSoQlf028E8eGp";

#[derive(Deserialize, Debug)]
struct ApiResponse {
    adjusted: Option<bool>,
    status: String,
    ticker: String,
    results: Vec<ResultItem>,
}


#[derive(Deserialize, Debug)]
struct ResultItem {
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    t: i64,
    v: f64,
}

#[derive(Serialize, Debug, Clone)]
struct StockData {
    symbol: String,
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64, // Change the type of the volume field to f64
}

#[derive(Serialize, Debug)]
struct TechnicalAnalysis {
    sma: f64,
    ema: f64,
    rsi: f64,
}

async fn fetch_stock_data(symbol: &str) -> Result<Vec<StockData>, Box<dyn Error>> {
    let now = Utc::now();
    let to_date = now.format("%Y-%m-%d").to_string();
    let from_date = (now - Duration::from_std(std::time::Duration::from_secs(100 * 24 * 60 * 60))?).format("%Y-%m-%d").to_string();
    let url = format!("{}/{}/range/1/day/{}/{}?adjusted=true&apiKey={}", POLYGON_API_URL,symbol, from_date, to_date, POLYGON_API_KEY);

    let response: ApiResponse = reqwest::get(&url).await?.json().await?;
    
    // Check if the adjusted field is present and has the expected value
    if let Some(adjusted) = response.adjusted {
        if !adjusted {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "The API response is not adjusted")));
        }
    } else {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "The 'adjusted' field is missing in the API response")));
    }

    let stock_data: Vec<StockData> = response.results.into_iter().map(|result| {
        StockData {
            symbol: response.ticker.clone(),
            timestamp: match NaiveDateTime::from_timestamp_opt(result.t / 1000, 0) {
                Some(dt) => dt.format("%Y-%m-%d").to_string(),
                None => String::new(),
            },            
            open: result.o,
            high: result.h,
            low: result.l,
            close: result.c,
            volume: result.v,
        }
    }).collect();

    Ok(stock_data)
}


fn calculate_technical_analysis(stock_data: &[StockData]) -> TechnicalAnalysis {
    let mut sma = SimpleMovingAverage::new(14).unwrap();
    let mut ema = ExponentialMovingAverage::new(14).unwrap();
    let mut rsi = RelativeStrengthIndex::new(14).unwrap();

    for data in stock_data.iter() {
        sma.next(data.close);
        ema.next(data.close);
        rsi.next(data.close);
    }

    let sma_value = sma.next(stock_data.last().unwrap().close);
    let ema_value = ema.next(stock_data.last().unwrap().close);
    let rsi_value = rsi.next(stock_data.last().unwrap().close);

    TechnicalAnalysis {
        sma: sma_value,
        ema: ema_value,
        rsi: rsi_value,
    }
}

#[get("/stock/{symbol}")]
async fn stock_data_and_analysis(
    symbol: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    match fetch_stock_data(&symbol).await {
        Ok(stock_data) => {
            let technical_analysis = calculate_technical_analysis(&stock_data);

            let result = serde_json::json!({
                "stock_data": stock_data,
                "technical_analysis": technical_analysis,
            });

            Ok(HttpResponse::Ok().json(result))
        }
        Err(e) => {
            eprintln!("Error fetching stock data and analysis: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let server_ip = "0.0.0.0";
    let server_port = 8080;

    println!("Starting server on {}:{}", server_ip, server_port);

    HttpServer::new(|| App::new().service(stock_data_and_analysis))
        .bind((server_ip, server_port))?
        .run()
        .await
}

