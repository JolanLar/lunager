use reqwest::blocking::Client;
use rusqlite::Connection;

use super::{movie::Movie, radarr::Radarr, serie::Serie, sonarr::Sonarr};

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

    // get first overseerr from database
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

    // make a get request to overseerr
    fn reqwest_get(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let response = client.get(url).header("x-api-key", &self.api_key).send()?;
        
        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        response.text().map_err(|err| err.into())
    }

    fn convert_date_to_timestamp(&self, date: &str) -> i32 {
        let date = date.trim_end_matches("Z");
        let date = date.replace("T", " ");
        let date = chrono::NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S.000").unwrap();
        let date = date.timestamp() as i32;

        date
    }

    // get all movies from overseerr
    pub fn get_all_movies(&self, conn: &Connection) -> Result<Vec<Movie>, Box<dyn std::error::Error>> {
        let mut movies: Vec<Movie> = Vec::new();

        let url = format!("{}/api/v1/Media?take=5000", self.url);
        let response = self.reqwest_get(url.as_str())?;
        let json: serde_json::Value = serde_json::from_str(&response)?;
        
        for media in json["results"].as_array().unwrap() {
            // if the ratingKey is null, the movie is not in plex, so skip it
            if (media["ratingKey"].is_null() && media["ratingKey4k"].is_null()) || media["mediaType"].as_str().unwrap() != "movie" {
                continue;
            }

            // convert the date in "createdAt" to a unix timestamp
            let created_at = self.convert_date_to_timestamp(media["createdAt"].as_str().unwrap());
            
            movies.push(
                Movie { 
                    tmdb_id: media["tmdbId"].as_i64().unwrap() as i32, 
                    name: String::new(), 
                    path_hd: String::new(), 
                    path_4k: String::new(),
                    rating_key: media["ratingKey"].as_str().unwrap_or("").to_string(),
                    last_view: created_at, 
                    protected: false
                }
            );
        }

        Ok(movies)
    }

    
    pub fn get_all_series(&self, conn: &Connection) -> Result<Vec<Serie>, Box<dyn std::error::Error>> {
        let mut series: Vec<Serie> = Vec::new();

        let url = format!("{}/api/v1/Media?take=5000", self.url);
        let response = self.reqwest_get(url.as_str())?;
        let json: serde_json::Value = serde_json::from_str(&response)?;
        
        for media in json["results"].as_array().unwrap() {
            // if the ratingKey is null, the serie is not in plex, so skip it
            if (media["ratingKey"].is_null() && media["ratingKey4k"].is_null()) || media["mediaType"].as_str().unwrap() != "tv" {
                continue;
            }

            // convert the date in "createdAt" to a unix timestamp
            let created_at = self.convert_date_to_timestamp(media["createdAt"].as_str().unwrap());
            
            series.push(
                Serie { 
                    tvdb_id: media["tvdbId"].as_i64().unwrap() as i32, 
                    name: String::new(), 
                    path_hd: String::new(), 
                    path_4k: String::new(),
                    rating_key: media["ratingKey"].as_str().unwrap_or("").to_string(),
                    last_view: created_at, 
                    protected: false
                }
            );
        }

        Ok(series)
    }

    // get overseer movies and insert missing one into the database
    pub fn update_db_movies(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let db_movies = Movie::get_all(&conn)?;
        let overseerr_movies = self.get_all_movies(&conn)?;

        for overseerr_movie in overseerr_movies {
            if !db_movies.contains(&overseerr_movie) {
                overseerr_movie.save(&conn)?;
            }
        }

        Ok(())
    }

    // get overseer series and insert missing one into the database
    pub fn update_db_series(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let db_series = Serie::get_all(&conn)?;
        let overseerr_series = self.get_all_series(&conn)?;

        for overseerr_serie in overseerr_series {
            if !db_series.contains(&overseerr_serie) {
                overseerr_serie.save(&conn)?;
            }
        }

        Ok(())
    }

    // get radarrs configuration from overseerr
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


    // get sonarrs configuration from overseerr
    pub fn get_sonarrs(&self) -> Result<Vec<Sonarr>, Box<dyn std::error::Error>> {
        let mut sonarrs: Vec<Sonarr> = Vec::new();

        let response = &self.reqwest_get(format!("{}/api/v1/settings/sonarr", self.url).as_str())?;
        let json: serde_json::Value = serde_json::from_str(&response)?;

        for sonarr in json.as_array().unwrap() {
            sonarrs.push(Sonarr::new(
                sonarr["externalUrl"].as_str().unwrap(), 
                sonarr["apiKey"].as_str().unwrap(),
                sonarr["is4k"].as_bool().unwrap()
            ));
        }

        Ok(sonarrs)
    }
}