mod auth;
mod cache;
mod components;
mod db;
mod logging;
mod routes;

use std::path::PathBuf;
use logging::RouterBuilderLoggingExt;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Router, TrustedProxies},
};

#[tokio::main]
async fn main() {
    let db_path_str = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "library.db".to_string());
    let db_path = PathBuf::from(&db_path_str);
    let pool_size = std::env::var("DATABASE_POOL_SIZE")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(10);
    let pool = db::DbPool::new(&db_path, pool_size).expect("Failed to initialize database pool");
    let cache = cache::AppCache::new(1000, 600); // 1,000 max capacity, 10 min TTL

    println!("Database initialized and verified at {}", db_path.display());

    let router = Router::builder()
        .trusted_proxies(
            TrustedProxies::new()
                .networks(["127.0.0.1", "::1", "10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"])
                .nearest(1),
        )
        .cookies()
        .logging()
        .app_context(pool)
        .app_context(cache)
        .layout(routes::layout::root_layout)
        // Auth Routes
        .page(routes::auth::login_page)
        .route(routes::auth::login_action)
        .route(routes::auth::logout_action)
        // Dashboard
        .page(routes::dashboard::dashboard_page)
        // Books Catalog
        .page(routes::books::books_page)
        .route(routes::books::books_table)
        .route(routes::books::book_modal_new)
        .route(routes::books::book_create_action)
        .route(routes::books::book_modal_edit)
        .route(routes::books::book_edit_action)
        .route(routes::books::book_delete_action)
        .route(routes::books::books_select_options)
        // Patrons & Members
        .page(routes::members::members_page)
        .route(routes::members::members_table)
        .route(routes::members::member_toggle_status)
        .route(routes::members::member_modal_new)
        .route(routes::members::member_create_action)
        .route(routes::members::member_drawer)
        .route(routes::members::members_select_options)
        // Circulation & Loans
        .page(routes::loans::loans_page)
        .route(routes::loans::loans_table)
        .route(routes::loans::loan_return_action)
        .route(routes::loans::loan_modal_new)
        .route(routes::loans::loan_issue_action)
        .assets(AssetBundle::load().expect("Failed to load bundled assets. Run 'topcoat asset bundle' first."))
        .build();

    println!("Starting Topcoat Library Application server...");
    topcoat::start(router).await.unwrap();
}
