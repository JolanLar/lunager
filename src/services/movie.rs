use rusqlite::{Connection, params};

use super::serie::Serie;

#[derive(Debug)]
pub struct Movie {
    pub tmdb_id: i32,
    pub name: String,
    pub path_hd: String,
    pub path_4k: String,
    pub rating_key: String,
    pub last_view: i32,
    pub protected: bool
}

impl Movie {
    pub fn save(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute("
            REPLACE INTO movie (tmdb_id, name, path_hd, path_4k, rating_key, last_view, protected)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        ", params![
            &self.tmdb_id,
            &self.name,
            &self.path_hd,
            &self.path_4k,
            &self.rating_key,
            &self.last_view,
            &self.protected,
        ])?;
        Ok(())
    }

    // create static function that returns all database movies
    pub fn get_all(conn: &Connection) -> Result<Vec<Movie>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("
            SELECT tmdb_id, name, path_hd, path_4k, rating_key, last_view, protected
            FROM movie
        ")?;

        let movies_iter = stmt.query_map([], |row| {
            Ok(Movie {
                tmdb_id: row.get(0)?,
                name: row.get(1)?,
                path_hd: row.get(2)?,
                path_4k: row.get(3)?,
                rating_key: row.get(4)?,
                last_view: row.get(5)?,
                protected: row.get(6)?
            })
        })?;

        let mut movies = Vec::new();
        for movie in movies_iter {
            movies.push(movie?);
        }

        Ok(movies)
    }

    // create from_radarr_json function
    pub fn from_radarr_json(json: &serde_json::Value, is4k: bool) -> Self {
        let mut movie = Movie {
            tmdb_id: json["tmdbId"].as_i64().unwrap() as i32,
            name: json["title"].as_str().unwrap().to_string(),
            path_hd: String::new(),
            path_4k: String::new(),
            rating_key: String::new(),
            last_view: 0,
            protected: false
        };
        if is4k {
            movie.path_4k = json["rootFolderPath"].as_str().unwrap().to_string();
        } else {
            movie.path_hd = json["rootFolderPath"].as_str().unwrap().to_string();
        }
        movie
    }

    // function to get a movie by his title
    pub fn get_by_title(conn: &Connection, title: &str) -> Result<Movie, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("
            SELECT tmdb_id, name, path_hd, path_4k, rating_key, last_view, protected
            FROM movie
            WHERE trim(lower(name)) = trim(lower(?))
        ")?;

        let mut movie_iter = stmt.query_map([title], |row| {
            Ok(Movie {
                tmdb_id: row.get(0)?,
                name: row.get(1)?,
                path_hd: row.get(2)?,
                path_4k: row.get(3)?,
                rating_key: row.get(4)?,
                last_view: row.get(5)?,
                protected: row.get(6)?
            })
        })?;

        if let Some(result) = movie_iter.next() {
            result.map_err(|err| err.into())
        } else {
            Err(format!("Movie {} not found", title).into())
        }
    }

    pub fn get_movies_to_delete(conn: &Connection, last_view: i32) -> Result<Vec<Movie>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("
            SELECT tmdb_id, name, path_hd, path_4k, rating_key, last_view, protected
            FROM movie
            WHERE last_view < ?
        ")?;

        let mut movie_iter = stmt.query_map([last_view], |row| {
            Ok(Movie {
                tmdb_id: row.get(0)?,
                name: row.get(1)?,
                path_hd: row.get(2)?,
                path_4k: row.get(3)?,
                rating_key: row.get(4)?,
                last_view: row.get(5)?,
                protected: row.get(6)?
            })
        })?;

        let mut movies = Vec::new();
        while let Some(result) = movie_iter.next() {
            movies.push(result?);
        }

        Ok(movies)
    }

    pub fn get_by_rating_key(conn: &Connection, rating_key: &str) -> Result<Movie, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("
            SELECT tmdb_id, name, path_hd, path_4k, rating_key, last_view, protected
            FROM movie
            WHERE rating_key = ?
        ")?;

        let mut movie_iter = stmt.query_map([rating_key], |row| {
            let movie = Movie {
                tmdb_id: row.get(0)?,
                name: row.get(1)?,
                path_hd: row.get(2)?,
                path_4k: row.get(3)?,
                rating_key: row.get(4)?,
                last_view: row.get(5)?,
                protected: row.get(6)?
            };
            Ok(movie)
        })?;

        if let Some(result) = movie_iter.next() {
            result.map_err(|err| err.into())
        } else {
            Err(format!("Movie not found for the rating key : {}", rating_key).into())
        }
    }
}

// add partial_eq trait to Movie struct
impl PartialEq for Movie {
    fn eq(&self, other: &Self) -> bool {
        self.tmdb_id == other.tmdb_id
    }
}