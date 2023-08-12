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
    is4k: bool
}

impl Radarr {
    pub fn new(conn: &Connection, url: &str, api_key: &str, is4k: bool) -> Self {
        conn.execute("REPLACE INTO radarr (url, api_key, is4k) VALUES (?, ?, ?)", params![url, api_key, is4k]).unwrap();

        Radarr {
            id: i32::from(conn.last_insert_rowid() as i32),
            url: url.to_string(),
            api_key: api_key.to_string(),
            is4k: is4k
        }
    }

    fn reqwest_get(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client.get(url).header("X-API-KEY", &self.api_key).send()?;
        
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

        let mut paths: Vec<RadarrPath> = Vec::new();

        for root_folder in root_folders_iter {
            let disk = match Disk::get_by_free_space(root_folder.freeSpace) {
                Ok(disk) => disk,
                Err(_) => Disk::new(&conn, root_folder.freeSpace)
            };
            paths.push(RadarrPath::new(&conn, self.id, &root_folder.path, disk.get_id()));
        };

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

    pub fn get_all_movies(&self) -> Result<Vec<Movie>, Box<dyn std::error::Error>> {
        let mut movies: Vec<Movie> = Vec::new();
        let url = format!("{}/api/v3/movie", self.url);
        let response = self.reqwest_get(url.as_str())?;
        let json: serde_json::Value = serde_json::from_str(&response)?;

        let movies_json = json.as_array().unwrap();
        for movie_json in movies_json {
            if movie_json["tmdbId"].is_null() || movie_json["hasFile"].as_bool().unwrap() == false {
                continue;
            }
            movies.push(Movie::from_radarr_json(movie_json, self.is4k));
        }

        Ok(movies)
    }

    pub fn update_db_movies(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let radarr_movies = self.get_all_movies()?;
        let mut db_movies = Movie::get_all(&conn)?;

        let mut quantity_created = 0;
        let mut quantity_updated = 0;

        for radarr_movie in radarr_movies {
            if let Some(db_movie) = db_movies.iter_mut().find(|db_movie| db_movie.tmdb_id == radarr_movie.tmdb_id) {
                let mut changed = false;

                // update name if changed
                if db_movie.name != radarr_movie.name {
                    db_movie.name = radarr_movie.name;
                    changed = true;
                }

                // if is4k = true, update path_4k if changed
                if self.is4k && db_movie.path_4k != radarr_movie.path_4k {
                    db_movie.path_4k = radarr_movie.path_4k;
                    changed = true;
                }

                // if is4k = false, update path_hd if changed
                if !self.is4k && db_movie.path_hd != radarr_movie.path_hd {
                    db_movie.path_hd = radarr_movie.path_hd;
                    changed = true;
                }

                // if changed, update db
                if changed {
                    db_movie.save(&conn)?;
                    quantity_updated += 1;
                }
            } else {
                radarr_movie.save(&conn)?;
                quantity_created += 1;
            }
        }

        println!("Created movies : {}", quantity_created);
        println!("Updated movies : {}", quantity_updated);
        Ok(())
    }
}