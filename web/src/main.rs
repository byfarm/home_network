use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web::Data};
use askama::Template;
use async_sqlite::{JournalMode, Pool, PoolBuilder, rusqlite::Connection};
use polars::prelude::*;
use serde::Serialize;

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    stdev: f32,
    mean: f32,
    min: f32,
    max: f32,
    chart_data: String,
}

#[derive(Serialize)]
struct ChartData<TM, T> {
    x: Vec<TM>,
    y: Vec<T>,
}

fn get_data(conn: &Connection) -> async_sqlite::rusqlite::Result<(Vec<String>, Vec<f32>)> {
    let mut stmt = conn.prepare("SELECT timestamp, value FROM data")?;
    let mut rows = stmt.query([])?;
    let mut timestamps: Vec<String> = Vec::new();
    let mut values: Vec<f32> = Vec::new();
    while let Some(row) = rows.next()? {
        timestamps.push(row.get(0)?);
        values.push(row.get(1)?);
    }

    Ok((timestamps, values))
}

#[get("/")]
async fn index(pool: actix_web::web::Data<Pool>) -> impl Responder {
    let (timestamps, values) = pool.conn(get_data).await.unwrap();

    let df: DataFrame = df!("x" => &timestamps, "y" => &values).unwrap();

    let clone_df = df.clone().lazy().select([col("y")]).collect().unwrap();
    let yseries = clone_df.column("y").unwrap().f32().unwrap();

    let stdev = yseries.std(0).unwrap() as f32;

    let mean = yseries.mean().unwrap() as f32;
    
    let min = yseries.min().unwrap() as f32;
    let max = yseries.max().unwrap() as f32;

    let card_data_raw = ChartData {
        x: timestamps,
        y: values,
    };

    let chart_data = serde_json::to_string(&card_data_raw).unwrap();

    let body = IndexTemplate {
        stdev,
        mean,
        chart_data,
        min,
        max
    };
    HttpResponse::Ok().body(body.render().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PoolBuilder::new()
        .path("../server/db.sqlite3")
        .journal_mode(JournalMode::Wal)
        .open()
        .await
        .expect("Unable to open new database pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(
                Files::new("/static", "./static")
                    .prefer_utf8(true)
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .service(index)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

#[cfg(test)]
mod test {
    use super::*;
    #[actix_web::test]
    async fn test_query() {
        let pool = PoolBuilder::new()
            .path("../server/db.sqlite3")
            .journal_mode(JournalMode::Wal)
            .open()
            .await
            .expect("Unable to open new database pool.");
        let (timestamps, _) = pool.conn(get_data).await.unwrap();
        println!("{:?}", timestamps);
        // assert_eq!(1, 2)
    }
}
