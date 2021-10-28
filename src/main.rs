use comfy_table::Table;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth};

// async fn main() -> Result<()> {
//     let mut oauth = SpotifyOAuth::default()
//         .scope("playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private")
//         .build();

//     if let Some(token_info) = get_token(&mut oauth).await {
//         let client_credential = SpotifyClientCredentials::default()
//             .token_info(token_info)
//             .build();

//         let spotify = Spotify::default()
//             .client_credentials_manager(client_credential)
//             .build();

//         let playlists = spotify.current_user_playlists(None, None).await?;
//         let mut table = Table::new();
//         table.set_header(vec!["ID", "Name", "Songs", "Public"]);
//         for playlist in playlists.items.into_iter() {
//             table.add_row(vec![
//                 playlist.id,
//                 playlist.name,
//                 playlist.tracks["total"].to_string(),
//                 if playlist.public.unwrap() {
//                     "Yes".to_string()
//                 } else {
//                     "No".to_string()
//                 },
//             ]);
//         }
//         println!("{}", table);
//     } else {
//         println!("auth failed");
//     }

//     Ok(())
// }

fn main() {
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
