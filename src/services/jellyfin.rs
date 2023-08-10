use std::result;

use reqwest::blocking::Client;
use rusqlite::{Connection, Result};
use serde_json::Value;
use super::{movie::Movie, serie::Serie};

#[derive(Debug)]
pub struct Jellyfin {
    id: i32,
    pub url: String,
    pub api_key: String
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
    
    // create reqwest_post function
    fn reqwest_post(&self, url: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client.post(url).header("X-Emby-Token", &self.api_key).header("Content-Type", "application/json").body(body.to_string()).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        response.text().map_err(|err| err.into())
    }

    fn update_media_activity(&self, query: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.url, "/user_usage_stats/submit_custom_query");
        let body = format!("{{\"CustomQueryString\":\"{}\"}}", query);
        let response = self.reqwest_post(url.as_str(), body.as_str())?;
        
        // get movies from results array [title, last played] in the response
        let response_json: serde_json::Value = serde_json::from_str(&response)?;
        // convert response_json["results"] to array and return error if it's null
        let results = match response_json["results"].as_array() {
            Some(results) => results,
            None => return Err("No results found".into())
        };

        Ok(results.clone())
    }

    // create function to get the movies activity of the last 2 months using "/user_usage_stats/submit_custom_query" path
    pub fn update_movies_activity(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let query = "SELECT IFNULL(NULLIF(SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')), ''), ItemName) ItemName, strftime('%s', strftime('%s', max(DateCreated)), 'unixepoch') lastView FROM PlaybackActivity WHERE SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')) == '' GROUP BY IFNULL(NULLIF(SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')), ''), ItemName)";
        let results = self.update_media_activity(query)?;
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
            let last_played = self.clean_api_timestamp(&result[1]);
            if movie.last_view > last_played {
                continue;
            }
            movie.last_view = last_played;

            movie.save(&conn)?;
        };
        Ok(())
    }

    pub fn update_series_activity(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let query = "SELECT IFNULL(NULLIF(SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')), ''), ItemName) ItemName, strftime('%s', strftime('%s', max(DateCreated)), 'unixepoch') lastView FROM PlaybackActivity WHERE SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')) != '' GROUP BY IFNULL(NULLIF(SUBSTR(ItemName, 0, INSTR(ItemName, ' - ')), ''), ItemName)";
        let results = self.update_media_activity(query)?;
        for result in results {
            if result[1].is_null() {
                continue;
            }
            let title = result[0].as_str().unwrap();

            let mut serie = match Serie::get_by_title(&conn, title) {
                Ok(serie) => serie,
                Err(err) => {
                    println!("Error: {}", err);
                    continue;
                }
            };

            // Set last view timestamp
            let last_played = self.clean_api_timestamp(&result[1]);
            if serie.last_view > last_played {
                continue;
            }
            serie.last_view = last_played;

            serie.save(&conn)?;
        };
        Ok(())
    }

    fn clean_api_timestamp(&self, timestamp: &Value) -> i32 {
        let mut timestamp = timestamp.to_string();
        let mut timestamp_chars = timestamp.chars();
        timestamp_chars.next();
        timestamp_chars.next_back();
        timestamp = timestamp_chars.as_str().to_string();
        timestamp.parse::<i32>().unwrap()
    }
}