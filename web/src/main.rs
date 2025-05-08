use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    body: &'a str,
}

#[get("/")]
async fn index() -> impl Responder {
    let body = IndexTemplate {
        body: "Hello World",
    };
    HttpResponse::Ok().body(body.render().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}
