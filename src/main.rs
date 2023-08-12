mod database;
use database::initialize_database;
use rusqlite::Connection;
use services::jellyfin::Jellyfin;
use services::overseerr::Overseerr;

use crate::services::movie::Movie;
use crate::services::serie::Serie;
use crate::services::tautulli::Tautulli;

mod services;

fn main() {
    // Initialize the sqlite database
    match initialize_database() {
        Ok(_) => (),
        Err(err) => println!("{:?}", err)
    };

    let conn = Connection::open("data.db").unwrap();

    println!("====================Overseerr====================");
    let overseerr = Overseerr::get_first(&conn);
    match overseerr.update_db_movies(&conn) {
        Ok(_) => (),
        Err(err) => println!("{:?}", err)
    };
    match overseerr.update_db_series(&conn) {
        Ok(_) => (),
        Err(err) => println!("{:?}", err)
    };
    println!("====================Overseerr====================");
    println!("");
    println!("====================Radarr====================");
    let radarrs = match overseerr.get_radarrs(&conn) {
        Ok(radarrs) => {
            println!("Successfully got radarrs from overseerr");
            radarrs
        },
        Err(err) => {
            println!("{:?}", err);
            Vec::new()
        }
    };

    for radarr in radarrs {
        match radarr.populate_paths(&conn) {
            Ok(_) => println!("Successfully populated paths for radarr"),
            Err(err) => println!("{:?}", err)
        }
        match radarr.update_db_movies(&conn) {
            Ok(_) => (),
            Err(err) => println!("{:?}", err)
        };
    }
    println!("====================Radarr====================");
    println!("");
    println!("====================Sonarr====================");
    let sonarrs = match overseerr.get_sonarrs() {
        Ok(sonarrs) => {
            println!("Successfully got sonarrs from overseerr");
            sonarrs
        },
        Err(err) => {
            println!("{:?}", err);
            Vec::new()
        }
    };

    for sonarr in sonarrs {
        match sonarr.populate_paths(&conn) {
            Ok(_) => println!("Successfully populated paths for sonarr"),
            Err(err) => println!("{:?}", err)
        }
        match sonarr.update_db_series(&conn) {
            Ok(_) => (),
            Err(err) => println!("{:?}", err)
        };
    }
    println!("====================Sonarr====================");
    println!("");
    println!("====================Jellyfin====================");
    for jellyfin in Jellyfin::get_all(&conn) {
        match jellyfin.update_movies_activity(&conn) {
            Ok(_) => (),
            Err(err) => println!("{:?}", err)
        };
        match jellyfin.update_series_activity(&conn) {
            Ok(_) => (),
            Err(err) => println!("{:?}", err)
        };
    }
    println!("====================Jellyfin====================");
    println!("");
    println!("====================Tautulli====================");
    let tautullis = match Tautulli::get_all(&conn) {
        Ok(tautullis) => {
            println!("Successfully got tautullis from overseerr");
            tautullis
        },
        Err(err) => {
            println!("{:?}", err);
            Vec::new()
        }
    };
    for mut tautulli in tautullis {
        match tautulli.update_medias_activity(&conn) {
            Ok(_) => (),
            Err(err) => println!("{:?}", err)
        };
    }
    println!("====================Tautulli====================");
    println!("");
    println!("====================Movies to delete====================");
    // Get three months ago date as a timestamp
    let three_months_ago = (chrono::Utc::now().timestamp() - 60 * 60 * 24 * 30 * 3) as i32;
    let movies_to_delete = match Movie::get_movies_to_delete(&conn, three_months_ago) {
        Ok(movies_to_delete) => {
            println!("Successfully got movies to delete");
            movies_to_delete
        },
        Err(err) => {
            println!("{:?}", err);
            Vec::new()
        }
    };
    println!("Quantity founded : {:?}", movies_to_delete.len());
    println!("====================Movies to delete====================");
    println!("");
    println!("====================Series to delete====================");
    // Get three months ago date as a timestamp
    let three_months_ago = (chrono::Utc::now().timestamp() - 60 * 60 * 24 * 30 * 3) as i32;
    let series_to_delete = match Serie::get_series_to_delete(&conn, three_months_ago) {
        Ok(series_to_delete) => {
            println!("Successfully got series to delete");
            series_to_delete
        },
        Err(err) => {
            println!("{:?}", err);
            Vec::new()
        }
    };
    println!("Quantity founded : {:?}", series_to_delete.len());
    println!("====================Series to delete====================");
}
