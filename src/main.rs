use comfy_table::Table;
use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
use rspotify::util::get_token;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let mut oauth = SpotifyOAuth::default()
        .scope("playlist-read-private playlist-read-collaborative playlist-modify-public playlist-modify-private")
        .build();

    if let Some(token_info) = get_token(&mut oauth).await {
        let client_credential = SpotifyClientCredentials::default()
            .token_info(token_info)
            .build();

        let spotify = Spotify::default()
            .client_credentials_manager(client_credential)
            .build();

        let playlists = spotify.current_user_playlists(None, None).await?;
        let mut table = Table::new();
        table.set_header(vec!["ID", "Name", "Songs", "Public"]);
        for (id, playlist) in playlists.items.into_iter().enumerate() {
            table.add_row(vec![
                id.to_string(),
                playlist.name,
                playlist.tracks["total"].to_string(),
                if playlist.public.unwrap() {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
            ]);
        }
        println!("{}", table);
    } else {
        println!("auth failed");
    }

    Ok(())
}
