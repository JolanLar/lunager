use super::disk::Disk;
use rusqlite::{Connection, Result, params};

pub struct SonarrPath {
    sonarr_id: i32,
    path: String,
    disk_id : i32
}

impl SonarrPath {
    pub fn new(conn: &Connection, sonarr_id: i32, path: &str, disk_id: i32) -> Self {
        conn.execute("REPLACE INTO sonarr_path (sonarr_id, path, disk_id) VALUES (?, ?, ?)", params![sonarr_id, path, disk_id]).unwrap();
        Self {
            sonarr_id: sonarr_id,
            path: path.to_string(),
            disk_id: disk_id
        }
    }
}

#[derive(Debug)]
pub struct RadarrPath {
    radarr_id: i32,
    path: String,
    disk_id : i32
}

impl RadarrPath {
    pub fn new(conn: &Connection, radarr_id: i32, path: &str, disk_id: i32) -> Self {
        conn.execute("REPLACE INTO radarr_path (radarr_id, path, disk_id) VALUES (?, ?, ?)", params![radarr_id, path, disk_id]).unwrap();

        Self {
            radarr_id: radarr_id,
            path: path.to_string(),
            disk_id: disk_id
        }
    }
}

trait GetDisk {
    fn get_disk(&self) -> Result<Disk, Box<dyn std::error::Error>>;
}

impl GetDisk for SonarrPath {
    fn get_disk(&self) -> Result<Disk, Box<dyn std::error::Error>> {
        Disk::get_by_id(self.disk_id)
    }
}