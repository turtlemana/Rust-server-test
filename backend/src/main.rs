use actix_web::{web, App, HttpServer, HttpResponse, get, Responder};
use actix_web::middleware::DefaultHeaders;
use sqlx::MySqlPool;
use dotenv::dotenv;
use std::env;
use serde_json::json;



#[get("/api/{ticker}")]
async fn handler(pool: web::Data<MySqlPool>, ticker: web::Path<String>) -> impl Responder {
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

#[get("/api/getCoinTicker")]
async fn handler_detail_info(pool: web::Data<MySqlPool>) -> impl Responder {
let query = format!(
    "SELECT al.ITEM_CD_DL as ticker
    FROM RMS.ALL_ASSETS al 
    WHERE CAT = 'Crypto'
    ORDER BY al.TRADE_VALUE DESC",
);
let rows = sqlx::query_as::<_, (String,)>(query.as_str())
        .fetch_one(pool.as_ref())
        .await;

match rows {
    Ok(result) => HttpResponse::Ok().json(result),
    Err(err) => {
        println!("Error: {:?}", err);
        HttpResponse::InternalServerError().json(json!({ "message": "Can't find data" }))
    }
}
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to create pool");

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
            .service(handler)
            .service(handler_detail_info)
    })
    .bind("127.0.0.1:7878")?
    .run()
    .await
}
