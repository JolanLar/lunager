use rusqlite::{Connection, params};

#[derive(Debug)]
pub struct Movie {
    tmdb_id: i32,
    name: String,
    path_hd: String,
    path_4k: String,
    rating_key: String,
    last_view: i32,
    protected: bool
}

impl Movie {
    pub fn new(
        conn: &Connection,
        tmdb_id: i32,
        name: Option<String>,
        path_hd: Option<String>,
        path_4k: Option<String>,
        rating_key: Option<String>,
        last_view: Option<i32>,
        protected: Option<bool>,
    ) -> Self {
        let movie = Movie {
            tmdb_id: tmdb_id,
            name: name.unwrap_or(String::new()),
            path_hd: path_hd.unwrap_or(String::new()),
            path_4k: path_4k.unwrap_or(String::new()),
            rating_key: rating_key.unwrap_or(String::new()),
            last_view: last_view.unwrap_or(i32::MIN),
            protected: protected.unwrap_or(false),
        };
        movie.save(&conn);
        movie
    }

    pub fn save(&self, conn: &Connection) {
        conn.execute("
            REPLACE INTO movie (tmdb_id, name, radarr_id, path, rating_key, last_view, protected)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        ", params![
            &self.tmdb_id,
            &self.name,
            &self.radarr_id,
            &self.path,
            &self.rating_key,
            &self.last_view,
            &self.protected,
        ]).unwrap();
        if self.tmdb_id == 736526 as i32 {
            println!("Movie saved: {:?}", self);
        }
    }

    pub fn get_by_tmdb_id(conn: &Connection, tmdb_id: i32) -> Result<Movie, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("
            SELECT tmdb_id, name, radarr_id, path, rating_key, last_view, protected
            FROM movie
            WHERE tmdb_id = ?
        ")?;

        let mut movie_iter = stmt.query_map([tmdb_id], |row| {
            Ok(Movie {
                tmdb_id: row.get(0)?,
                name: row.get(1)?,
                radarr_id: row.get(2)?,
                path: row.get(3)?,
                rating_key: row.get(4)?,
                last_view: row.get(5)?,
                protected: row.get(6)?
            })
        })?;

        if let Some(result) = movie_iter.next() {
            result.map_err(|err| err.into())
        } else {
            Err("Movie not found".into())
        }
    }

    // create static function that returns all database movies
    pub fn get_all(conn: &Connection) -> Result<Vec<Movie>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("
            SELECT tmdb_id, name, radarr_id, path, rating_key, last_view, protected
            FROM movie
        ")?;

        let movies_iter = stmt.query_map([], |row| {
            Ok(Movie {
                tmdb_id: row.get(0)?,
                name: row.get(1)?,
                radarr_id: row.get(2)?,
                path: row.get(3)?,
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
    pub fn from_radarr_json(conn: &Connection, radarr_id: i32, json: &serde_json::Value) -> Self {
        let movie = match Movie::get_by_tmdb_id(&conn, json["tmdbId"].as_i64().unwrap() as i32) {
            Ok(mut movie) => {
                movie.name = json["title"].as_str().unwrap().to_string();
                movie.path = json["rootFolderPath"].as_str().unwrap().to_string();
                movie.radarr_id = radarr_id;
                movie
            },
            Err(_) => Movie {
                    tmdb_id: json["tmdbId"].as_i64().unwrap() as i32,
                    name: json["title"].as_str().unwrap().to_string(),
                    radarr_id: radarr_id,
                    path: json["rootFolderPath"].as_str().unwrap().to_string(),
                    rating_key: String::new(),
                    last_view: 0,
                    protected: false
                }
            };
        movie.save(&conn);
        movie
    }

    // function to get a movie by his title
    pub fn get_by_title(conn: &Connection, title: &str) -> Result<Movie, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("
            SELECT tmdb_id, name, radarr_id, path, rating_key, last_view, protected
            FROM movie
            WHERE lower(name) = lower(?)
        ")?;

        let mut movie_iter = stmt.query_map([title], |row| {
            Ok(Movie {
                tmdb_id: row.get(0)?,
                name: row.get(1)?,
                radarr_id: row.get(2)?,
                path: row.get(3)?,
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

    pub fn get_last_view(&self) -> i32 {
        self.last_view
    }

    pub fn set_last_view(&mut self, last_view: i32) {
        self.last_view = last_view;
    }
}