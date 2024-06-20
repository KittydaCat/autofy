use std::fs;
use serde_json::Value;
use async_recursion::async_recursion;

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

    async fn request_get(&self, url: &str) -> Value {

        self.client.get(url)
            .bearer_auth(&self.auth)
            .send().await.unwrap()
            .json::<Value>().await.unwrap()

    }

    // fn get_auth

    async fn get_public_playlist(&self, id: &str) -> Playlist {

        let json = self.request_get(&format!("https://api.spotify.com/v1/playlists/{id}")).await;


        // println!("{}", json.to_string());

        let name = json.as_object().unwrap().get("name").unwrap().as_str().unwrap().to_string();

        let tracks_json = json.as_object().unwrap().get("tracks").unwrap();

        let tracks = self.get_tracks(tracks_json).await;

        // let total_items = dbg!(tracks_json.get("total").unwrap()).as_u64().unwrap();
        //
        // let mut tracks_items_json = tracks_json.get("items").unwrap()
        //     .as_array().unwrap();
        //
        // let mut tracks = Vec::new();
        //
        // loop {
        //     for track_json in tracks_items_json {
        //         if let Some(track) = track_json.as_object().unwrap()
        //             .get("track").unwrap().as_object() {
        //             let artists = track.get("artists").unwrap()
        //                 .as_array().unwrap()
        //                 .iter()
        //                 .map(|x| x.as_object().unwrap().get("name").unwrap().as_str().unwrap().to_string())
        //                 .collect();
        //
        //             tracks.push(Track {
        //                 name: track.get("name").unwrap().as_str().unwrap().to_string(),
        //                 artists,
        //             })
        //         }
        //     }
        //
        //     if tracks.len() as u64 != total_items {
        //
        //         response = self.client.get(dbg!(tracks_json.get("next").unwrap().as_str()
        //             .expect("if this fails it means there is less items than expected in the vec but there isnt another page")))
        //             .bearer_auth(&self.auth).send().await.unwrap();
        //
        //         json = response.json::<Value>().await.unwrap();
        //
        //         // tracks_json = dbg!(json.as_object().unwrap()).get("tracks").unwrap()
        //         //     .as_object().unwrap();
        //
        //         tracks_items_json = json.get("items").unwrap()
        //             .as_array().unwrap();
        //
        //     } else {
        //         break
        //     }
        //
        // }
        
        Playlist {
            name,
            tracks,
        }

    }

    #[async_recursion]
    async fn get_tracks(&self, tracks_json: &Value) -> Vec<Track> {

        let items_json = tracks_json.get("items").unwrap().as_array().unwrap();

        let mut tracks: Vec<_> = items_json.iter().filter_map(|track_json| {
            let track = track_json.get("track").unwrap().as_object()?;
            let artists = track.get("artists").unwrap()
                .as_array().unwrap()
                .iter()
                .map(|x| dbg!(dbg!(x).get("name").unwrap()).as_str().unwrap_or("Unknown Artist").to_string())
                .collect();

            Some(Track {
                name: track.get("name").unwrap().as_str().unwrap().to_string(),
                artists,
            })

        }).collect();

        if tracks.len() as u64 != dbg!(tracks_json.get("total").unwrap()).as_u64().unwrap() {

            if let Some(next) = tracks_json.get("next")
                .expect("if this fails it means there is less items than expected in the vec but there isnt another page")
                .as_str() {
                tracks.append(&mut self.get_tracks(&self.request_get(next).await).await)
            }

            else {
            //     why???

                panic!()
            }

        }

        tracks

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
