use rusqlite::{Connection, params};

use super::{movie::Movie, serie::Serie};

pub struct Tautulli {
    pub id: i32,
    pub url: String,
    pub api_key: String,
    pub history: Option<serde_json::Value>
}

impl Tautulli {
    pub fn new(conn: &Connection, url: &str, api_key: &str) -> Tautulli {
        conn.execute("REPLACE INTO tautulli (url, api_key) VALUES (?, ?)", params![url, api_key]).unwrap();
        let tautulli = Tautulli {
            id: i32::from(conn.last_insert_rowid() as i32),
            url: url.to_string(),
            api_key: api_key.to_string(),
            history: None
        };
        tautulli
    }

    pub fn save(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute("
            REPLACE INTO tautulli (url, api_key)
            VALUES (?1, ?2)",
            params![self.url, self.api_key]
        )?;
        Ok(())
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Tautulli>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("
            SELECT id, url, api_key
            FROM tautulli
        ")?;
        let tautullis = stmt.query_map([], |row| {
            Ok(Tautulli {
                id: row.get(0)?,
                url: row.get(1)?,
                api_key: row.get(2)?,
                history: None
            })
        })?;

        let mut tautullis_vec = Vec::new();
        for tautulli in tautullis {
            tautullis_vec.push(tautulli?);
        }
        Ok(tautullis_vec)
    }

    fn reqwest_get(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let response = client.get(url).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        response.text().map_err(|err| err.into())
    }

    fn get_history(&mut self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        if self.history.is_none() {
            self.update_history()?;
        }
        Ok(self.history.clone().unwrap_or_default())
    }

    fn update_history(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}{}{}", self.url, "/api/v2?cmd=get_history&length=500000&apikey=", self.api_key);
        let response = self.reqwest_get(url.as_str())?;
        let results: serde_json::Value = serde_json::from_str(&response)?;
        println!("Quantity founded : {:?}", results["response"]["data"]["data"].as_array().unwrap().len());
        self.history = Some(results);
        Ok(())
    }

    pub fn update_medias_activity(&mut self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let history = self.get_history()?;
        let mut quantity_updated = 0;
        for activity in history["response"]["data"]["data"].as_array().unwrap() {
            // get the media from the rating key
            let rating_key = activity["grandparent_rating_key"].as_i64().or(activity["parent_rating_key"].as_i64()).or(activity["rating_key"].as_i64()).unwrap().to_string();
            if activity["media_type"] == "movie" {
                let mut movie = match Movie::get_by_rating_key(conn, rating_key.as_str()) {
                    Ok(movie) => movie,
                    Err(_) => continue
                };
            
                // update the movie last_view
                let last_view = activity["date"].as_i64().unwrap() as i32;
                if movie.last_view < last_view{
                    movie.last_view = last_view;
                    movie.save(conn)?;
                    quantity_updated += 1;
                }
            } else {
                let mut serie = match Serie::get_by_rating_key(conn, rating_key.as_str()) {
                    Ok(serie) => serie,
                    Err(_) => continue
                };
            
                // update the serie last_view
                let last_view = activity["date"].as_i64().unwrap() as i32;
                if serie.last_view < last_view{
                    serie.last_view = last_view;
                    serie.save(conn)?;
                    quantity_updated += 1;
                }
            }
        }
        println!("Updated medias : {}", quantity_updated);
        Ok(())
    }
}

trait Or: Sized {
    fn or(self, other: Self) -> Self;
}

impl<'a> Or for &'a str {
    fn or(self, other: &'a str) -> &'a str {
        if self.is_empty() { other } else { self }
    }
}