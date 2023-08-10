use super::{disk::Disk, movie::Movie};
use super::path::RadarrPath;
use reqwest::blocking::Client;
use serde::Deserialize;
use rusqlite::{Connection, Result, params};

#[derive(Deserialize)]
struct RootFolder {
    path: String,
    freeSpace: u64,
    id: i32
}

#[derive(Debug)]
pub struct Radarr {
    id: i32,
    url: String,
    api_key: String,
    paths: Vec<RadarrPath>,
    is4k: bool
}

impl Radarr {
    pub fn new(url: &str, api_key: &str, is4k: bool) -> Self {
        let conn = Connection::open("data.db").unwrap();

        conn.execute("REPLACE INTO radarr (url, api_key, is4k) VALUES (?, ?, ?)", params![url, api_key, is4k]).unwrap();

        Radarr {
            id: i32::from(conn.last_insert_rowid() as i32),
            url: url.to_string(),
            api_key: api_key.to_string(),
            paths: vec![],
            is4k: is4k
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

    fn reqwest_get(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client.get(url).header("X-API-KEY", &self.api_key).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        response.text().map_err(|err| err.into())
    }

    pub fn populate_paths(&mut self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.url, "/api/v3/rootfolder");
        let body = self.reqwest_get(url.as_str())?;

        let root_folders: Vec<RootFolder> = serde_json::from_str(&body)?;
        let root_folders_iter = root_folders.iter();

        let mut paths: Vec<RadarrPath> = Vec::new();

        for root_folder in root_folders_iter {
            let disk = match Disk::get_by_free_space(root_folder.freeSpace) {
                Ok(disk) => disk,
                Err(_) => Disk::new(&conn, root_folder.freeSpace)
            };
            paths.push(RadarrPath::new(&conn, self.id, &root_folder.path, disk.get_id()));
        };

        self.paths = paths;

        Ok(())
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Radarr>> {
        let mut radarrs: Vec<Radarr> = Vec::new();

        let mut stmt = conn.prepare("SELECT id, url, api_key, is4k FROM radarr")?;
        let radarrs_iter = stmt.query_map([], |row| {
            let mut radarr = Radarr {
                id: row.get(0)?,
                url: row.get(1)?,
                api_key: row.get(2)?,
                paths: Vec::new(),
                is4k: row.get(3)?
            };
            radarr.populate_paths(&conn).unwrap();
            Ok(radarr)
        })?;
        for radarr in radarrs_iter {
            radarrs.push(radarr?)
        }

        Ok(radarrs)
    }

    pub fn get_all_movies(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/v3/movie", self.url);
        let response = self.reqwest_get(url.as_str())?;
        let json: serde_json::Value = serde_json::from_str(&response)?;

        let movies_json = json.as_array().unwrap();
        for movie_json in movies_json {
            if movie_json["tmdbId"].is_null() || movie_json["hasFile"].as_bool().unwrap() == false {
                continue;
            }
            Movie::from_radarr_json(&conn, self.id, movie_json);
        }

        Ok(())
    }
}