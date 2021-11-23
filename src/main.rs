use comfy_table::Table;
use rand::prelude::SliceRandom;
use rspotify::model::PlaylistId;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt)]
enum Opts {
    Playlists,
    List {
        id: ListId,
    },
    Export {
        id: ListId,
    },
    Merge {
        dest: ListId,
        src: ListId,
        src_rest: Vec<ListId>,
    },
    Sort {
        id: ListId,
        kind: Sort,
        #[structopt(default_value = "ascending")]
        direction: Direction,
    },
    Duplicates {
        id: ListId,
    },
}

enum Sort {
    Random,
    Energy,
}

impl FromStr for Sort {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "random" => Ok(Sort::Random),
            "energy" => Ok(Sort::Energy),
            _ => Err("unknown sort kind"),
        }
    }
}

enum Direction {
    Ascending,
    Descending,
}

impl FromStr for Direction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ascending" => Ok(Direction::Ascending),
            "descending" => Ok(Direction::Descending),
            _ => Err("unknown sort direction"),
        }
    }
}

#[derive(Debug)]
struct ListId {
    id: String,
}

impl FromStr for ListId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = match s.strip_prefix("https://open.spotify.com/playlist/") {
            Some(id_and_query) => match id_and_query.find('?') {
                Some(query_location) => &id_and_query[..query_location],
                None => id_and_query,
            },
            None => s,
        };

        if id.chars().all(|c| c.is_alphanumeric()) && id.len() == 22 {
            return Ok(ListId { id: id.to_string() });
        }

        Err("invalid playlist id")
    }
}

fn main() {
    let args = Opts::from_args();

    // Enabling automatic token refreshing in the config
    let mut config = Config::default();
    config.token_refreshing = true;

    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!(
        "playlist-read-private",
        "playlist-read-collaborative",
        "playlist-modify-public",
        "playlist-modify-private"
    ))
    .unwrap();

    let mut spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    // Obtaining the access token
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).unwrap();

    match args {
        Opts::Playlists => {
            // Typical iteration, no extra boilerplate needed.
            let stream = spotify.current_user_playlists();

            let mut table = Table::new();
            table.set_header(vec!["ID", "Name", "Songs", "Public"]);
            for item in stream {
                let playlist = item.unwrap();
                table.add_row(vec![
                    playlist
                        .id
                        .to_string()
                        .strip_prefix("spotify:playlist:")
                        .unwrap()
                        .to_string(),
                    playlist.name,
                    playlist.tracks.total.to_string(),
                    if playlist.public.unwrap() {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    },
                ]);
            }
            println!("{}", table);
        }
        Opts::List { id } => {
            let playlist_id = PlaylistId::from_str(&id.id).unwrap();
            let playlist_items = spotify.playlist_items(&playlist_id, None, None);
            let mut table = Table::new();
            table.set_header(vec!["Name", "Artist", "Album", "Length", "ID"]);
            for item in playlist_items {
                let track = item.unwrap().track.unwrap();
                match track {
                    rspotify::model::PlayableItem::Track(track) => {
                        table.add_row(vec![
                            track.name,
                            track.artists[0].name.clone(),
                            track.album.name,
                            track.duration.as_secs().to_string(),
                            track.id.to_string(),
                        ]);
                    }
                    rspotify::model::PlayableItem::Episode(episode) => {
                        table.add_row(vec![
                            episode.name,
                            episode.show.publisher,
                            episode.show.name,
                            episode.duration.as_secs().to_string(),
                            episode.id.to_string(),
                        ]);
                    }
                }
            }
            println!("{}", table);
        }
        Opts::Export { id } => todo!(),
        Opts::Merge {
            dest,
            src,
            src_rest,
        } => todo!(),
        Opts::Sort {
            id,
            kind,
            direction,
        } => {
            let playlist_id = PlaylistId::from_str(&id.id).unwrap();
            let playlist_items = spotify.playlist_items(&playlist_id, None, None);
            let mut items = Vec::new();
            for playlist_item in playlist_items {
                let track = playlist_item.unwrap().track.unwrap();
                items.push(track);
            }
            let mut playable_ids = Vec::new();
            let mut track_ids = Vec::new();
            for item in &items {
                if let rspotify::model::PlayableItem::Track(t) = item {
                    track_ids.push((&t.id, item.id()));
                }
                playable_ids.push(item.id());
            }
            match kind {
                Sort::Random => {
                    playable_ids.shuffle(&mut rand::thread_rng());
                    push_playlist(playable_ids, &spotify, &playlist_id);
                }
                Sort::Energy => {
                    let mut energy = HashMap::new();
                    for chunk in track_ids.chunks(100) {
                        for track in spotify
                            .tracks_features(chunk.iter().map(|(id, _)| *id))
                            .unwrap()
                            .unwrap()
                        {
                            energy.insert(track.id, track.energy);
                        }
                    }
                    track_ids.sort_by(|(id_a, _), (id_b, _)| match direction {
                        Direction::Ascending => energy[*id_a].partial_cmp(&energy[*id_b]).unwrap(),
                        Direction::Descending => energy[*id_b].partial_cmp(&energy[*id_a]).unwrap(),
                    });
                    push_playlist_tracks(track_ids, spotify, playlist_id);
                }
            }
        }
        Opts::Duplicates { id } => {
            let playlist_id = PlaylistId::from_str(&id.id).unwrap();
            let playlist_items = spotify.playlist_items(&playlist_id, None, None);
            let playlist_items: Result<Vec<_>, _> = playlist_items.collect();
            let playlist_items = playlist_items.unwrap();
            let mut playlist_hashset = HashSet::new();
            for item in playlist_items {
                let playable_item = item.track.unwrap();
                let id = playable_item.id().id().to_string();
                if !playlist_hashset.insert(id) {
                    match playable_item {
                        rspotify::model::PlayableItem::Track(track) => {
                            println!("{}", track.name);
                        }
                        rspotify::model::PlayableItem::Episode(episode) => {
                            println!("{}", episode.name)
                        }
                    }
                }
            }
        }
    }
}

fn push_playlist(
    playable_ids: Vec<&dyn PlayableId>,
    spotify: &AuthCodeSpotify,
    playlist_id: &PlaylistId,
) {
    for (i, chunk) in playable_ids.chunks(100).enumerate() {
        if i == 0 {
            spotify
                .playlist_replace_items(playlist_id, chunk.iter().copied())
                .unwrap();
        } else {
            spotify
                .playlist_add_items(playlist_id, chunk.iter().copied(), None)
                .unwrap();
        }
    }
}

fn push_playlist_tracks(
    track_ids: Vec<(&rspotify::model::TrackId, &dyn PlayableId)>,
    spotify: AuthCodeSpotify,
    playlist_id: PlaylistId,
) {
    for (i, chunk) in track_ids.chunks(100).enumerate() {
        if i == 0 {
            spotify
                .playlist_replace_items(&playlist_id, chunk.iter().map(|(_, id)| *id))
                .unwrap();
        } else {
            spotify
                .playlist_add_items(&playlist_id, chunk.iter().map(|(_, id)| *id), None)
                .unwrap();
        }
    }
}
