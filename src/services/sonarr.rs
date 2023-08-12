use super::{disk::Disk, serie::Serie};
use super::path::SonarrPath;
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
pub struct Sonarr {
    id: i32,
    url: String,
    api_key: String,
    is4k: bool
}

impl Sonarr {
    pub fn new(url: &str, api_key: &str, is4k: bool) -> Self {
        let conn = Connection::open("data.db").unwrap();

        conn.execute("REPLACE INTO sonarr (url, api_key, is4k) VALUES (?, ?, ?)", params![url, api_key, is4k]).unwrap();

        Sonarr {
            id: i32::from(conn.last_insert_rowid() as i32),
            url: url.to_string(),
            api_key: api_key.to_string(),
            is4k: is4k
        }
    }

    fn reqwest_get(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client.get(url).header("X-Api-Key", &self.api_key).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        response.text().map_err(|err| err.into())
    }

    pub fn populate_paths(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.url, "/api/v3/rootfolder");
        let body = self.reqwest_get(url.as_str())?;

        let root_folders: Vec<RootFolder> = serde_json::from_str(&body)?;
        let root_folders_iter = root_folders.iter();

        let mut paths: Vec<SonarrPath> = Vec::new();

        for root_folder in root_folders_iter {
            let disk = match Disk::get_by_free_space(root_folder.freeSpace) {
                Ok(disk) => disk,
                Err(_) => Disk::new(&conn, root_folder.freeSpace)
            };
            paths.push(SonarrPath::new(&conn, self.id, &root_folder.path, disk.get_id()));
        };

        Ok(())
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Sonarr>> {
        let mut sonarrs: Vec<Sonarr> = Vec::new();

        let mut stmt = conn.prepare("SELECT id, url, api_key, is4k FROM sonarr")?;
        let sonarrs_iter = stmt.query_map([], |row| {
            let mut sonarr = Sonarr {
                id: row.get(0)?,
                url: row.get(1)?,
                api_key: row.get(2)?,
                is4k: row.get(3)?
            };
            sonarr.populate_paths(&conn).unwrap();
            Ok(sonarr)
        })?;
        for sonarr in sonarrs_iter {
            sonarrs.push(sonarr?)
        }

        Ok(sonarrs)
    }

    pub fn get_all_series(&self) -> Result<Vec<Serie>, Box<dyn std::error::Error>> {
        let mut series: Vec<Serie> = Vec::new();
        let url = format!("{}/api/v3/series", self.url);
        let response = self.reqwest_get(url.as_str())?;
        let json: serde_json::Value = serde_json::from_str(&response)?;

        let series_json = json.as_array().unwrap();
        for serie_json in series_json {
            if serie_json["tvdbId"].is_null() || serie_json["statistics"]["episodeFileCount"].as_i64().unwrap() == 0 {
                continue;
            }
            series.push(Serie::from_sonarr_json(serie_json, self.is4k));
        }

        Ok(series)
    }

    pub fn update_db_series(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let sonarr_series = self.get_all_series()?;
        let mut db_series = Serie::get_all(&conn)?;

        let mut quantity_created = 0;
        let mut quantity_updated = 0;

        for sonarr_serie in sonarr_series {
            if let Some(db_serie) = db_series.iter_mut().find(|db_serie| db_serie.tvdb_id == sonarr_serie.tvdb_id) {
                let mut changed = false;

                // update name if changed
                if db_serie.name != sonarr_serie.name {
                    db_serie.name = sonarr_serie.name.clone();
                    changed = true;
                }

                // if is4k = true, update path_4k if changed
                if self.is4k && db_serie.path_4k != sonarr_serie.path_4k {
                    db_serie.path_4k = sonarr_serie.path_4k.clone();
                    changed = true;
                }

                // if is4k = false, update path_hd if changed
                if !self.is4k && db_serie.path_hd != sonarr_serie.path_hd {
                    db_serie.path_hd = sonarr_serie.path_hd.clone();
                    changed = true;
                }

                // if changed, update db
                if changed {
                    db_serie.save(&conn)?;
                    quantity_updated += 1;
                }
            } else {
                sonarr_serie.save(&conn)?;
                quantity_created += 1;
            }
        }

        println!("Created series : {}", quantity_created);
        println!("Updated series : {}", quantity_updated);
        Ok(())
    }
}