use std::fs;
use serde_json::Value;

#[derive(Clone, Debug)]
struct Track {
    name: String,
    artists: Vec<String>,
}

#[derive(Clone, Debug)]
struct Playlist {
    name: String,
    // discription: String,
    tracks: Vec<Track>
}
#[derive(Clone, Debug)]
struct Client {
    client: reqwest::Client,
    id: String,
    secret: String,
    auth: String,
}

impl Client {

    async fn new() -> Client {

        let file = fs::read_to_string("secrets.txt").unwrap();
        let mut strs = file.split("\r\n");

        let id = strs.next().unwrap().parse().unwrap();
        let secret = strs.next().unwrap().parse().unwrap();

        let client = reqwest::Client::new();

        let response = client.post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("grant_type=client_credentials&client_id={id}&client_secret={secret}"))
            .send()
            .await.unwrap();

        let auth = dbg!(response.json::<Value>().await.unwrap())
            .as_object().unwrap()
            .get("access_token").unwrap()
            .as_str().unwrap().to_string();

        Client {
            client,
            id,
            secret,
            auth,
        }

    }

    // fn get_auth

    async fn get_public_playlist(&self, name: &str) -> Playlist {

        let response = self.client.get(format!("https://api.spotify.com/v1/playlists/{name}"))
            .bearer_auth(&self.auth).send().await.unwrap();

        let json = response.json::<Value>().await.unwrap();

        // println!("{}", json.to_string());

        let name = json.as_object().unwrap().get("name").unwrap().as_str().unwrap().to_string();

        let tracks_json = json.as_object().unwrap().get("tracks").unwrap()
            .as_object().unwrap()
            .get("items").unwrap()
            .as_array().unwrap();

        let tracks = tracks_json.iter().filter_map(|track_json| {

            let track = track_json.as_object().unwrap()
                .get("track").unwrap()
                .as_object()?;

            let artists = track.get("artists").unwrap()
                .as_array().unwrap()
                .iter()
                .map(|x| x.as_object().unwrap().get("name").unwrap().as_str().unwrap().to_string())
                .collect();

            Some(Track {
                name: track.get("name").unwrap().as_str().unwrap().to_string(),
                artists,
            })
        }).collect::<Vec<Track>>();
        
        Playlist {
            name,
            tracks,
        }

    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {

        let client = dbg!(Client::new().await);

        dbg!(dbg!(client.get_public_playlist("1VhaQKU3TNk1EFfOtCtCbD").await).tracks.len());
    }
}
