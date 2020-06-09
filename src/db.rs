use crate::{
    lfn::LevelFileName,
    se::{SerPolygon, SerPolygonOwner, SerVertex},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{read_dir, File},
    io::{BufReader, BufWriter},
};

#[derive(Serialize, Deserialize)]
pub struct Db {
    pub polygons: Vec<SerPolygonOwner>,
    pub levels: HashSet<LevelFileName>,
    pub tag: Option<String>,
}

impl Db {
    pub fn new() -> Db {
        Db {
            polygons: Vec::new(),
            levels: HashSet::new(),
            tag: None,
        }
    }

    pub fn load_database(db_path: &str) -> std::io::Result<Db> {
        let f = File::open(db_path)?;
        let r = BufReader::new(f);
        match bincode::deserialize_from::<BufReader<File>, Db>(r) {
            Ok(db) => Ok(db),
            Err(_) => std::io::Result::Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
        }
    }

    pub fn write_database(&self, db_path: &str) -> std::result::Result<(), ()> {
        let f = File::create(db_path).unwrap();
        let w = BufWriter::new(f);
        match bincode::serialize_into::<BufWriter<File>, Db>(w, self) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    pub fn from_directory(dir_path: &str) -> std::io::Result<Db> {
        let mut failed_levs: Vec<LevelFileName> = Vec::new();
        let mut n_failed = 0;
        let mut n_polygons = 0;

        let mut database = Db::new();

        println!("\nLOADED      FAILED      POLYGONS");

        for (i, de) in read_dir(dir_path)?.enumerate() {
            let entry = de.unwrap();
            let path = entry.path();
            if match path.extension() {
                Some(ext) => ext == "lev",
                None => false,
            } {
                let level_file_name = LevelFileName::from_osstr((&*path).file_name().unwrap());
                if let Ok(mut new_polygons) = SerPolygonOwner::from_level_path(&*path) {
                    database.polygons.append(&mut new_polygons);
                    database.levels.insert(level_file_name);
                    n_polygons = database.polygons.len();
                } else {
                    failed_levs.push(level_file_name);
                    n_failed += 1;
                }
                print!("\r{:6}      {:6}      {:8}", i, n_failed, n_polygons);
            }
        }

        if failed_levs.len() > 0 {
            println!("\n\nCorrupt levels:");
            for failed_lev in failed_levs {
                println!("  {}", failed_lev);
            }
        } else {
            print! {"\n"};
        }
        print! {"\n"};

        Ok(database)
    }

    pub fn combine(&mut self, other: &mut Db) {
        let mut new_levels: HashSet<LevelFileName> = HashSet::new();

        while let Some(polygon) = other.polygons.pop() {
            if !self.levels.contains(&polygon.file_name) {
                if !new_levels.contains(&polygon.file_name) {
                    new_levels.insert(polygon.file_name.to_owned());
                }

                self.polygons.push(polygon);
            } else {
                println! {"Database already contains {}, skipping it.", polygon.file_name};
            }
        }

        for new_level in new_levels {
            self.levels.insert(new_level);
        }
    }
}
