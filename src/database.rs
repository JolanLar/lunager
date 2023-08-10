use rusqlite::{Connection, Result};

pub fn initialize_database() -> Result<()> {
    let conn = Connection::open("data.db")?;

    // Create radarr, sonarr, jellyseerr and overseerr tables
    let services  = vec!["jellyseerr", "overseerr"];
    for service in &services {
        conn.execute(&format!("
            CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT,
                url TEXT,
                api_key TEXT
            )", service),
            []
        )?;
    }


    let services  = vec!["radarr", "sonarr"];
    for service in &services {
        conn.execute(&format!("
            CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT,
                url TEXT,
                api_key TEXT,
                is4k INTEGER
            )", service),
            []
        )?;
    }

    // Create disk table
    conn.execute("
        CREATE TABLE IF NOT EXISTS disk (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            free_space INTEGER
        )",
        []
    )?;

    // Create radarr path table
    conn.execute("
        CREATE TABLE IF NOT EXISTS radarr_path (
            radarr_id INTEGER,
            path TEXT,
            disk_id INTEGER,
            PRIMARY KEY (radarr_id, path),
            FOREIGN KEY(radarr_id) REFERENCES radarr(id),
            FOREIGN KEY(disk_id) REFERENCES disk(id)
        )", 
        []
    )?;

    // Create sonarr path table
    conn.execute("
        CREATE TABLE IF NOT EXISTS sonarr_path (
            sonarr_id INTEGER,
            path TEXT,
            disk_id INTEGER,
            PRIMARY KEY (sonarr_id, path),
            FOREIGN KEY(sonarr_id) REFERENCES sonarr(id),
            FOREIGN KEY(disk_id) REFERENCES disk(id)
        )", 
        []
    )?;

    // Create movie table
    conn.execute("
        CREATE TABLE IF NOT EXISTS movie (
            tmdb_id INTEGER PRIMARY KEY,
            name TEXT,
            radarr_id INTERGER,
            path TEXT,
            rating_key TEXT,
            last_view INTERGER,
            protected INTEGER,
            FOREIGN KEY(radarr_id) REFERENCES radarr(id),
            FOREIGN KEY(path) REFERENCES radarr_disk(path)
        )", 
        []
    )?;


    // Create serie table
    conn.execute("
        CREATE TABLE IF NOT EXISTS serie (
            tmdb_id INTEGER PRIMARY KEY,
            name TEXT,
            sonarr_id INTERGER,
            path TEXT,
            rating_key TEXT,
            last_view INTERGER,
            protected INTEGER,
            FOREIGN KEY(sonarr_id) REFERENCES sonarr(id),
            FOREIGN KEY(path) REFERENCES sonarr_disk(path)
        )", 
        []
    )?;

    // Create jellyfin table
    conn.execute("
        CREATE TABLE IF NOT EXISTS jellyfin (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT,
            api_key TEXT
        )", 
        []
    )?;

    match conn.close() {
        Ok(_) => (),
        Err((_, err)) => println!("{}", err)
    };

    Ok(())
}