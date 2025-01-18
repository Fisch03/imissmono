use config::Config;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Artworks {
    artworks: Vec<Artwork>,
    artists: Vec<Artist>,
}

#[derive(Debug, Clone)]
pub struct Artwork {
    artist_id: usize,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub username: String,
    pub twitter: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArtistConfig {
    twitter: Option<String>,
    artworks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ArtworkConfig {
    artists: HashMap<String, ArtistConfig>,
}

impl From<ArtworkConfig> for Artworks {
    fn from(config: ArtworkConfig) -> Self {
        let mut artworks = Vec::new();
        let mut artists = Vec::new();

        for (id, (artist, artist_config)) in config.artists.into_iter().enumerate() {
            let artist = Artist {
                username: artist.clone(),
                twitter: artist_config.twitter,
            };
            for path in artist_config.artworks {
                artworks.push(Artwork {
                    artist_id: id,
                    path,
                });
            }

            artists.push(artist);
        }

        Self { artworks, artists }
    }
}

impl Artworks {
    pub fn load() -> Self {
        let artworks = Config::builder()
            .add_source(config::File::with_name("config/art"))
            .build()
            .expect("failed to load art config");

        let artworks: ArtworkConfig = artworks
            .try_deserialize()
            .expect("failed to deserialize art config");

        artworks.into()
    }

    pub fn get_random(&self) -> Option<(&Artwork, &Artist)> {
        let artwork = self.artworks.choose(&mut rand::thread_rng())?;
        let artist = self.artists.get(artwork.artist_id)?;
        Some((artwork, artist))
    }
}
