use comfy_table::Table;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth};
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
    },
}

enum Sort {
    Random,
}

impl FromStr for Sort {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "random" => Ok(Sort::Random),
            _ => Err("unknown sort kind"),
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
        Opts::List { id } => todo!(),
        Opts::Export { id } => todo!(),
        Opts::Merge {
            dest,
            src,
            src_rest,
        } => todo!(),
        Opts::Sort { id, kind } => todo!(),
    }
}
