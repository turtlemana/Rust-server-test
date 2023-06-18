use actix_web::{web, App, HttpServer, HttpResponse, get, Responder};
use actix_web::middleware::DefaultHeaders;
use sqlx::MySqlPool;
use dotenv::dotenv;
use std::env;
use serde_json::json;
use sqlx::FromRow;
use serde::Serialize;
use serde::Deserialize;
use std::collections::HashMap;


#[derive(FromRow, Serialize, Deserialize)]
pub struct Ticker {
    ticker: String,
}


#[derive(FromRow, Serialize, Deserialize)]
pub struct SearchResult {
    HR_ITEM_NM: Option<String>,
    ITEM_CD_DL: Option<String>,
    ITEM_ENG_NM: Option<String>,
    ITEM_KR_NM: Option<String>,
    WTHR_KR_DL: Option<String>,
    WTHR_ENG_DL: Option<String>,
    WTHR_ENG_NM: Option<String>,
    CVaR_LV: Option<String>,
    CVaR_LV_KR: Option<String>,
    CVaRNTS: Option<f64>,
    EXP_CVaRNTS: Option<f64>,
    ADJ_CLOSE: Option<f64>,
    ADJ_CLOSE_USD: Option<f64>,
    LV_DSCP_KR: Option<String>,
    LV_DSCP_ENG: Option<String>,
}
#[get("/api/search")]
async fn search(pool: web::Data<MySqlPool>, search: web::Query<HashMap<String, String>>) -> impl Responder {
    let search_string = search.get("search").unwrap_or(&String::from("")).clone();
    // Clean the search string
    let cleaned_search = search_string.trim().replace("  ", " ");
    
    let query = format!(
        "SELECT ITEM_KR_NM, HR_ITEM_NM, LV_DSCP_KR, LV_DSCP_ENG, ITEM_CD_DL, ITEM_ENG_NM, CVaR_LV, WTHR_ENG_NM, WTHR_ENG_DL, CVaRNTS, EXP_CVaRNTS, ADJ_CLOSE, ADJ_CLOSE_USD, WTHR_KR_DL, CVaR_LV_KR
        FROM RMS.ALL_ASSETS
        WHERE 1=1
        {condition}
        ORDER BY CASE WHEN LOC = 'Korea (South)' OR CAT='Crypto' OR ITEM_ENG_NM IN ('Apple Inc','Netflix Inc','Meta Platforn Inc Class A','Nvdia Corp','Microsoft Corp','Amazon Com Inc','Alphabet Inc Class A','Tesla Inc','Taiwan Semiconductor Manufacturing') THEN 0 ELSE 1 END, TRADE_VALUE DESC",
        condition = if !cleaned_search.is_empty() {
            format!("AND MATCH(ITEM_CD_DL, ITEM_ENG_NM,ITEM_KR_NM) AGAINST('{}' IN BOOLEAN MODE)", cleaned_search)
        } else {
            String::from("AND null")
        },
    );

    let rows: Result<Vec<SearchResult>, _> = sqlx::query_as(&query)
        .fetch_all(&**pool)
        .await;

    match rows {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            eprintln!("Database error: {:?}", e);  // This will print the error to stderr
            HttpResponse::InternalServerError().json(json!({ "message": "Can't find data" }))
        },
    }
}


#[get("/api/{ticker}")]
async fn detail_chart(pool: web::Data<MySqlPool>, ticker: web::Path<String>) -> impl Responder {
    let query = format!(
        "SELECT CHART
        FROM RMS.CHART_TMP 
        WHERE ITEM_CD_DL = '{}' AND
        CHART_TP=0",
        ticker
    );

    let rows = sqlx::query_as::<_, (String,)>(query.as_str())
        .fetch_one(pool.as_ref())
        .await;

    match rows {
        Ok((chart,)) => HttpResponse::Ok().json(json!({ "chart": chart })),
        Err(_) => HttpResponse::InternalServerError().json(json!({ "message": "Can't find data" })),
    }
}




#[get("/api/allTickers")]
async fn coin_tickers(pool: web::Data<MySqlPool>) -> impl Responder {
    let query = "SELECT al.ITEM_CD_DL as ticker FROM RMS.ALL_ASSETS al WHERE CAT = 'Crypto' ORDER BY al.TRADE_VALUE DESC";

    let rows: Result<Vec<Ticker>, _> = sqlx::query_as(query)
        .fetch_all(&**pool)
        .await;
    match rows {
        Ok(tickers) => HttpResponse::Ok().json(tickers),
        Err(e) => {
            eprintln!("Database error: {:?}", e);  // This will print the error to stderr
            HttpResponse::InternalServerError().json(json!({ "message": format!("Can't find data: {}", e) }))
        },
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to create pool");

    let test_query = "SELECT 1";
    match sqlx::query(test_query).execute(&pool).await {
        Ok(_) => println!("Database connected successfully"),
        Err(e) => eprintln!("Failed to connect to database: {:?}", e),
    }

    HttpServer::new(move || {
        App::new()
    .wrap(
    DefaultHeaders::new()
        .add(("Access-Control-Allow-Origin", "http://localhost:3000"))
        .add(("Access-Control-Allow-Methods", "GET"))
        .add(("Access-Control-Allow-Headers", "authorization, accept"))
        .add(("Access-Control-Allow-Headers", "content-type"))
        .add(("Access-Control-Max-Age", "3600"))
)
            .app_data(web::Data::new(pool.clone()))
            .service(search)
            .service(coin_tickers)
            .service(detail_chart)
        
    })
    .bind("127.0.0.1:7878")?
    .run()
    .await
}