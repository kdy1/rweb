#![cfg(not(feature = "openapi"))]

use futures::lock::Mutex;
use rweb::*;
use std::sync::Arc;

#[derive(Clone)]
struct Db {
    items: Arc<Mutex<Vec<String>>>,
}

#[get("/")]
async fn index(#[data] db: Db) -> Result<String, Rejection> {
    let items = db.items.lock().await;

    Ok(items.len().to_string())
}
