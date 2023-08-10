use rusqlite::{Connection, Result};

pub struct Disk {
    id: i32,
    free_space: u64
}

impl Disk {
    pub fn new(conn: &Connection, free_space: u64) -> Self {
        conn.execute("INSERT INTO disk (free_space) VALUES (?)", [free_space]).unwrap();

        Disk {
            id: i32::from(conn.last_insert_rowid() as i32),
            free_space: free_space
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_by_id(id: i32) -> Result<Disk, Box<dyn std::error::Error>> {
        let conn: Connection = Connection::open("data.db")?;

        let mut stmt = conn.prepare("SELECT id, free_space FROM disk WHERE id = ?")?;
        let mut disk_iter = stmt.query_map([id], |row| {
            Ok(Disk {
                id: row.get(0)?,
                free_space: row.get(0)?
            })
        })?;

        if let Some(result) = disk_iter.next() {
            result.map_err(|err| err.into())
        } else {
            Err("Disk not found".into())
        }
    }

    pub fn get_free_space(&self) -> u64 {
        self.free_space
    }

    pub fn set_free_space(&mut self, free_space: u64) {
        self.free_space = free_space;
    }

    pub fn get_by_free_space(free_space: u64) -> Result<Disk, Box<dyn std::error::Error>> {
        let conn: Connection = Connection::open("data.db")?;

        let mut stmt = conn.prepare("SELECT id, free_space FROM disk WHERE free_space = ?")?;
        let mut disk_iter = stmt.query_map([free_space], |row| {
            Ok(Disk {
                id: row.get(0)?,
                free_space: row.get(1)?
            })
        })?;
        
        if let Some(result) = disk_iter.next() {
            result.map_err(|err| err.into())
        } else {
            Err("Disk not found".into())
        }
    }
}