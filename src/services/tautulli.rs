use rusqlite::{Connection, params};

use super::movie::Movie;

pub struct Tautulli {
    pub id: i32,
    pub url: String,
    pub api_key: String
}

impl Tautulli {
    pub fn new(conn: &Connection, url: String, api_key: String) -> Tautulli {
        conn.execute("REPLACE INTO tautulli (url, api_key) VALUES (?, ?)", params![url, api_key]).unwrap();
        let tautulli = Tautulli {
            id: i32::from(conn.last_insert_rowid() as i32),
            url,
            api_key
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
                api_key: row.get(2)?
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

    pub fn update_movies_activity(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}{}{}", self.url, "/api/v2?cmd=get_history&length=500000&apikey=", self.api_key);
        let response = self.reqwest_get(url.as_str())?;
        let results: serde_json::Value = serde_json::from_str(&response)?;

        for result in results.as_array() {

        }

        Ok(())
    }
}