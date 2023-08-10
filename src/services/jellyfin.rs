use reqwest::blocking::Client;
use rusqlite::{Connection, Result};
use super::movie::Movie;

#[derive(Debug)]
pub struct Jellyfin {
    id: i32,
    url: String,
    api_key: String
}

impl Jellyfin {
    pub fn new(url: &str, api_key: &str) -> Self {
        let conn = Connection::open("data.db").unwrap();

        conn.execute("REPLACE INTO jellyfin (url, api_key) VALUES (?, ?)", [url, api_key]).unwrap();

        Jellyfin {
            id: i32::from(conn.last_insert_rowid() as i32),
            url: url.to_string(),
            api_key: api_key.to_string()
        }
    }

    pub fn set_token(&mut self, api_key: &str) {
        self.api_key = api_key.to_owned();
    }

    pub fn get_url(&self) -> &str {
        return self.url.as_str();
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_owned();
    }

    pub fn get_all(conn: &Connection) -> Vec<Jellyfin> {
        let mut stmt = conn.prepare("
            SELECT id, url, api_key
            FROM jellyfin
        ").unwrap();

        let mut jellyfin_iter = stmt.query_map([], |row| {
            Ok(Jellyfin {
                id: row.get(0)?,
                url: row.get(1)?,
                api_key: row.get(2)?
            })
        }).unwrap();

        let mut jellyfins = Vec::new();

        while let Some(result) = jellyfin_iter.next() {
            jellyfins.push(result.unwrap());
        }

        jellyfins
    }

    fn reqwest_get(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client.get(url).header("X-Emby-Token", &self.api_key).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        response.text().map_err(|err| err.into())
    }
    
    // create reqwest_post function
    fn reqwest_post(&self, url: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client.post(url).header("X-Emby-Token", &self.api_key).header("Content-Type", "application/json").body(body.to_string()).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        response.text().map_err(|err| err.into())
    }

    // create function to get the movies activity of the last 2 months using "/user_usage_stats/submit_custom_query" path
    pub fn get_movies_activity(&self, conn: &Connection, months: i32) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.url, "/emby/user_usage_stats/submit_custom_query");
        let body = format!("{{\"CustomQueryString\":\"SELECT DISTINCT IFNULL(NULLIF(SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')), ''), ItemName) ItemName, strftime('%s', strftime('%s', DateCreated), 'unixepoch') lastView FROM PlaybackActivity WHERE DateCreated > DATE('now', '-{} MONTH')  AND SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')) == '' GROUP BY IFNULL(NULLIF(SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')), ''), ItemName) ORDER BY DateCreated DESC\"}}", months);
        let response = self.reqwest_post(url.as_str(), body.as_str())?;
        
        // get movies from results array [title, last played] in the response
        let response_json: serde_json::Value = serde_json::from_str(&response)?;
        let results = response_json["results"].as_array().unwrap();
        for result in results {
            if result[1].is_null() {
                continue;
            }
            let title = result[0].as_str().unwrap();

            let mut movie = match Movie::get_by_title(&conn, title) {
                Ok(movie) => movie,
                Err(err) => {
                    println!("Error: {}", err);
                    continue;
                }
            };

            // Set last view timestamp
            let mut last_played = result[1].to_string();
            let mut last_played_chars = last_played.chars();
            last_played_chars.next();
            last_played_chars.next_back();
            last_played = last_played_chars.as_str().to_string();
            let last_played = last_played.parse::<i32>().unwrap();
            if movie.get_last_view() > last_played {
                continue;
            }
            movie.set_last_view(last_played);

            movie.save(&conn);
        };
        Ok(())
    }
}