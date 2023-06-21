use actix::{Actor, Addr};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, middleware, web};
use actix_web::web::Data;
use actix_web_actors::ws;
use rust_game_server_practice::game::Game;
use rust_game_server_practice::server::Session;

async fn ws(req: HttpRequest, stream: web::Payload, game: Data<Addr<Game>>) -> Result<HttpResponse, actix_web::Error> {
    ws::start(Session::new(game.get_ref().clone()), &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game = Game::default().start();

    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(game.clone()))
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(ws))
    })
        .bind(("0.0.0.0", 1111))?
        .run()
        .await
}
