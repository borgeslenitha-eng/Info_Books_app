mod models;
mod state;
mod handlers;
mod utils;

use actix_web::{web, App, HttpServer};
use actix_files::Files;
use tera::Tera;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 🔧 Carrega os templates com tratamento de erro
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => {
            println!("✅ Templates carregados com sucesso!");
            t
        }
        Err(e) => {
            eprintln!("❌ Erro ao carregar templates: {}", e);
            std::process::exit(1);
        }
    };

    // 🔹 Estado inicial (em memória)
    let app_state = AppState::new_with_sample_data();

    println!("🚀 Servidor rodando em: http://127.0.0.1:8080");

    // 🔹 Inicializa o servidor Actix
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(app_state.clone()))
            // serve arquivos estáticos (CSS, imagens, etc.)
            .service(Files::new("/static", "./static").show_files_listing())
            // rotas principais
            .route("/", web::get().to(handlers::index))
            .route("/login", web::get().to(handlers::login_page))
            .route("/api/login", web::post().to(handlers::do_login))
            .route("/register", web::get().to(handlers::register_page))
            .route("/api/register", web::post().to(handlers::do_register))
            .route("/book/{id}", web::get().to(handlers::book_page))
            .route("/api/rent", web::post().to(handlers::rent_book))
            .route("/api/return", web::post().to(handlers::return_book))
            .route("/admin", web::get().to(handlers::admin_dashboard))
    })
    .bind(("127.0.0.1", 8080))?   // ← fecha o HttpServer::new
    .run()
    .await
} 
