use reqwest::blocking::Client;
use rusqlite::Connection;

use super::{movie::Movie, radarr::Radarr};

#[derive(Debug)]
pub struct Overseerr {
    id: i32,
    url: String,
    api_key: String
}

impl Overseerr {
    pub fn new(url: &str, api_key: &str) -> Self {
        let conn = Connection::open("data.db").unwrap();

        conn.execute("INSERT INTO overseerr (url, api_key) VALUES (?, ?)", [url, api_key]).unwrap();

        Overseerr {
            id: i32::from(conn.last_insert_rowid() as i32),
            url: url.to_string(),
            api_key: api_key.to_string()
        }
    }

    // create function to get first overseerr from database
    pub fn get_first(conn: &Connection) -> Self {
        let mut stmt = conn.prepare("
            SELECT id, url, api_key
            FROM overseerr
            LIMIT 1
        ").unwrap();

        let mut overseerr_iter = stmt.query_map([], |row| {
            Ok(Overseerr {
                id: row.get(0)?,
                url: row.get(1)?,
                api_key: row.get(2)?
            })
        }).unwrap();

        let overseerr = overseerr_iter.next().unwrap().unwrap();

        overseerr
    }

    fn reqwest_get(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client.get(url).header("x-api-key", &self.api_key).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        response.text().map_err(|err| err.into())
    }

    // create function to get all movies from overseerr
    pub fn get_all_movies(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {

        let url = format!("{}/api/v1/Media?take=5000", self.url);
        let response = self.reqwest_get(url.as_str())?;
        let json: serde_json::Value = serde_json::from_str(&response)?;
        
        for media in json["results"].as_array().unwrap() {
            // if the ratingKey is null, the movie is not in plex, so skip it
            if (media["ratingKey"].is_null() && media["ratingKey4k"].is_null()) || media["mediaType"].as_str().unwrap() != "movie" {
                continue;
            }

            // convert the date in "createdAt" to a unix timestamp
            let created_at = media["createdAt"].as_str().unwrap().to_string();
            let created_at = created_at.trim_end_matches("Z");
            let created_at = created_at.replace("T", " ");
            let created_at = chrono::NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S.000").unwrap();
            let created_at = created_at.timestamp() as i32;

            Movie::new(
                &conn,
                media["tmdbId"].as_i64().unwrap() as i32,
                String::new(),
                0,
                String::new(),
                media["ratingKey"].as_str().unwrap_or("").to_string(),
                created_at,
                false
            );
        }

        Ok(())
    }

    pub fn get_radarrs(&self) -> Result<Vec<Radarr>, Box<dyn std::error::Error>> {

        let mut radarrs: Vec<Radarr> = Vec::new();

        let response = &self.reqwest_get(format!("{}/api/v1/settings/radarr", self.url).as_str())?;
        let json: serde_json::Value = serde_json::from_str(&response)?;

        for radarr in json.as_array().unwrap() {
            radarrs.push(Radarr::new(
                radarr["externalUrl"].as_str().unwrap(), 
                radarr["apiKey"].as_str().unwrap(),
                radarr["is4k"].as_bool().unwrap()
            ));
        }

        Ok(radarrs)
    }
}