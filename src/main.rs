use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, prelude::*, EnvFilter};

use axum::{extract::State, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use maud::{html, Markup, DOCTYPE};
use serde::Serialize;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

use holodex::model::{id::VideoId, *};

use imissmono::config;
use imissmono::db::*;

#[derive(Debug)]
struct AppState {
    stream_state: StreamState,
    last_update: Instant,
}

impl AppState {
    fn new() -> Self {
        let mut state = Self {
            last_update: Instant::now(),
            stream_state: StreamState::Unknown,
        };

        state.update();

        state
    }

    fn update(&mut self) {
        let _ = self.stream_state.update();
        self.last_update = Instant::now();
    }
}

#[derive(Debug, Clone, Serialize)]
enum StreamState {
    Live { video_id: VideoId },
    NotLive { last_live: DateTime<Utc> },
    Unknown,
}

impl StreamState {
    fn update(&mut self) -> Result<(), holodex::errors::Error> {
        let mut videos = config().holodex.videos()?;

        if let Some(live) = videos.iter().find(|v| v.status == VideoStatus::Live) {
            *self = StreamState::Live {
                video_id: live.id.clone(),
            };
        } else {
            videos.sort_unstable_by_key(|v| v.published_at);
            let last_live = videos.last().map(|v| v.published_at).flatten();

            if let Some(last_live) = last_live {
                *self = StreamState::NotLive { last_live };
            } else {
                *self = StreamState::Unknown;
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .expect("failed to create filter");
    let fmt_subscriber = tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .with_target(false)
        .with_filter(filter);

    let registry = tracing_subscriber::registry().with(fmt_subscriber);
    tracing::subscriber::set_global_default(registry).expect("failed to set subscriber");

    let serve_dir = ServeDir::new("static").append_index_html_on_directories(false);

    let router = Router::new()
        .route("/", get(main_page))
        // .route("/folder", get(folder_page))
        .route("/api", get(api))
        .with_state(Arc::new(Mutex::new(AppState::new())))
        .fallback_service(serve_dir)
        .layer(
            CompressionLayer::new()
                .gzip(true)
                .zstd(true)
                .br(true)
                .deflate(true),
        );

    let port = config().server.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}

async fn main_page() -> Markup {
    let db = db().await;
    let image: Image = sqlx::query_as("SELECT * FROM images ORDER BY RANDOM() LIMIT 1")
        .fetch_one(db)
        .await
        .expect("Failed to fetch image");

    let artist: Artist = sqlx::query_as("SELECT * FROM artists WHERE id = ?")
        .bind(image.artist)
        .fetch_one(db)
        .await
        .expect("Failed to fetch artist");

    html! {
        (DOCTYPE)
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title { "I MISS MONO" }
        link rel="stylesheet" href="style.css";
        body {
            div id="app" {
                div id="image" {
                    img src=(format!("images/{}", image.path)) {}
                    @if let Some(twitter) = artist.twitter {
                        a href=(format!("https://twitter.com/{}", twitter)) { (artist.username) }
                    } @else {
                        div id="artist" { (artist.username) }
                    }
                }
                div id="timer" {}
                div {
                    "heavily inspired by "
                    a href="https://imissfauna.com/" { "imissfauna.com" }
                    " - data provided by holodex - "
                    a href="https://github.com/Fisch03/imissmono" { "source" }
                    " - MILMM"
                }
            }
        }
        script src="https://cdnjs.cloudflare.com/ajax/libs/humanize-duration/3.32.1/humanize-duration.min.js" {}
        script src="https://www.youtube.com/player_api" {}
        script src="script.js" async {}
    }
}

async fn api(State(state): State<Arc<Mutex<AppState>>>) -> Json<StreamState> {
    let mut state = state.lock().unwrap();

    if state.last_update.elapsed().as_secs() > 60 {
        state.update();
    }

    Json(state.stream_state.clone())
}
