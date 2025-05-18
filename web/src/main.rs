use askama::Template;
use async_sqlite::{JournalMode, Pool, PoolBuilder, rusqlite::Connection};
// use polars::prelude::*;
use axum::{
    Router,
    extract::State,
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    // stdev: f32,
    // mean: f32,
    // min: f32,
    // max: f32,
    chart_data: String,
}

struct HtmlTemplate<T>(T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> axum::response::Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => panic!("{}", err),
        }
    }
}

#[derive(Serialize)]
struct ChartData<TM, T> {
    x: Vec<TM>,
    y: Vec<T>,
}

struct AppState {
    pool: Pool,
}

fn get_data(conn: &Connection) -> async_sqlite::rusqlite::Result<(Vec<String>, Vec<f32>)> {
    let mut stmt = conn.prepare("SELECT timestamp, value FROM data WHERE timestamp > CURRENT_TIMESTAMP;")?;
    let mut rows = stmt.query([])?;
    let mut timestamps: Vec<String> = Vec::new();
    let mut values: Vec<f32> = Vec::new();
    while let Some(row) = rows.next()? {
        timestamps.push(row.get(0)?);
        values.push(row.get(1)?);
    }

    Ok((timestamps, values))
}

async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (timestamps, values) = state.pool.conn(get_data).await.unwrap();

    // let df: DataFrame = df!("x" => &timestamps, "y" => &values).unwrap();
    //
    // let clone_df = df.clone().lazy().select([col("y")]).collect().unwrap();
    // let yseries = clone_df.column("y").unwrap().f32().unwrap();
    //
    // let stdev = yseries.std(0).unwrap() as f32;
    //
    // let mean = yseries.mean().unwrap() as f32;
    //
    // let min = yseries.min().unwrap() as f32;
    // let max = yseries.max().unwrap() as f32;

    let card_data_raw = ChartData {
        x: timestamps,
        y: values,
    };

    let chart_data = serde_json::to_string(&card_data_raw).unwrap();

    let body = IndexTemplate {
        // stdev,
        // mean,
        chart_data,
        // min,
        // max,
    };
    HtmlTemplate(body)
}

#[derive(Deserialize, Serialize)]
struct WsUpdate<TM, T> {
    date: TM,
    value: T,
}

async fn update(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|ws| handle_update(ws, state))
}

async fn handle_update(mut socket: WebSocket, state: Arc<AppState>) {
    let mut counter = 23;
    loop {
        let time: String = state
            .pool
            .conn(|conn| conn.query_row("SELECT CURRENT_TIMESTAMP;", [], |row| row.get(0)))
            .await
            .unwrap();
        let wire_data = WsUpdate {
            value: counter,
            date: time,
        };
        socket
            .send(serde_json::to_string(&wire_data).unwrap().into())
            .await
            .unwrap();
        counter += 1;
        if counter > 35 {
            counter = 0
        }
        sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() {
    let pool = PoolBuilder::new()
        .path("../server/db.sqlite3")
        .journal_mode(JournalMode::Wal)
        .open()
        .await
        .expect("Unable to open new database pool.");

    let static_files = tower_http::services::ServeDir::new("./static");

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(update))
        .with_state(Arc::new(AppState { pool }))
        .nest_service("/static", static_files);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
