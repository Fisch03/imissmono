use axum::{extract::State, routing::get, Router};
use chrono::{DateTime, Utc};
use maud::{html, Markup, DOCTYPE};
use serde::Serialize;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

use holodex::model::{id::VideoId, *};

use imissmono::{config, Artworks};

#[derive(Debug)]
struct AppState {
    stream_state: StreamState,
    artworks: Artworks,
}

impl AppState {
    fn new() -> Self {
        let mut state = Self {
            stream_state: StreamState::Unknown,
            artworks: Artworks::load(),
        };

        state.update();

        state
    }

    fn update(&mut self) {
        let _ = self.stream_state.update();
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
            videos.retain(|v| v.status == VideoStatus::Past);
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
    let serve_dir = ServeDir::new("static").append_index_html_on_directories(false);
    let serve_img = ServeDir::new("images").append_index_html_on_directories(false);

    let state = Arc::new(Mutex::new(AppState::new()));
    let router = Router::new()
        .route("/", get(main_page))
        .nest_service("/img", serve_img)
        // .route("/folder", get(folder_page))
        // .route("/api", get(api))
        .with_state(state.clone())
        .fallback_service(serve_dir)
        .layer(
            CompressionLayer::new()
                .gzip(true)
                .zstd(true)
                .br(true)
                .deflate(true),
        );

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            state.lock().unwrap().update();
            interval.tick().await;
        }
    });

    let port = config().server.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}

async fn main_page(State(state): State<Arc<Mutex<AppState>>>) -> Markup {
    let (stream_state, artwork, artist) = {
        let state = state.lock().unwrap();
        let stream_state = state.stream_state.clone();
        let (artwork, artist) = state.artworks.get_random().unwrap();

        (stream_state, artwork.clone(), artist.clone())
    };

    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            title { "I MISS MONO" }
            link rel="icon" type="image/png" href="/favicon/favicon-96x96.png" sizes="96x96";
            link rel="icon" type="image/svg+xml" href="/favicon/favicon.svg";
            link rel="shortcut icon" href="/favicon/favicon.ico";
            link rel="apple-touch-icon" sizes="180x180" href="/favicon/apple-touch-icon.png";
            meta name="apple-mobile-web-app-title" content="I MISS MONO";
            link rel="manifest" href="/favicon/site.webmanifest";
            link rel="stylesheet" href="style.css";
        }
        body data-state=(match stream_state {
            StreamState::Live { video_id } => format!("l {}", video_id),
            StreamState::NotLive { last_live } => format!("nl {}", last_live),
            StreamState::Unknown => "-".to_string(),
        }) {
            div id="app" {
                div id="image" {
                    img src=(format!("img/{}", artwork.path)) {}
                    span id="artist" {
                    "art by "
                        @if let Some(twitter) = &artist.twitter {
                            a href=(format!("https://twitter.com/{}", twitter)) { (artist.username) }
                        } @else {
                            (artist.username)
                        }
                    }
                }
                div {
                    div id="timer" {}
                    div id="reps" {
                        a href="https://www.youtube.com/@MonoMonet/streams" { "do your reps" }
                    }

                }
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
        script src="script.js" {}
    }
}
