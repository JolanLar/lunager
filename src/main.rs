mod database;
use database::initialize_database;
use rusqlite::Connection;
use services::jellyfin::{Jellyfin, self};
use services::overseerr::Overseerr;
use services::radarr::Radarr;
mod services;

fn main() {
    // Initialize the sqlite database
    match initialize_database() {
        Ok(_) => (),
        Err(err) => println!("{:?}", err)
    };

    let conn = Connection::open("data.db").unwrap();

    let overseerr = Overseerr::get_first(&conn);
    
    overseerr.get_all_movies(&conn).unwrap();

    let radarrs = overseerr.get_radarrs().unwrap();

    println!("{:?}", radarrs);

    for radarr in radarrs {
        radarr.get_all_movies(&conn).unwrap();
    }

    for jellyfin in Jellyfin::get_all(&conn) {
        jellyfin.get_movies_activity(&conn, 2).unwrap();
    }
}
