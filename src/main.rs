use bincode;
use clap::{crate_authors, crate_version, App, ArgGroup};
use elma::lev::{Level, Polygon, Top10Save};
use elma::Position;
use rand::{thread_rng, Rng};
use rand_distr::Normal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{read_dir, File, ReadDir};
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Serialize, Deserialize, Copy, Clone)]
struct SerVertex {
    x: f64,
    y: f64,
}
impl SerVertex {
    fn from_vertex(vertex: &Position<f64>) -> SerVertex {
        SerVertex {
            x: vertex.x,
            y: vertex.y,
        }
    }

    fn to_vertex(&self) -> Position<f64> {
        Position::new(self.x, self.y)
    }

    fn to_vertex_translate(&self, x: f64, y: f64) -> Position<f64> {
        Position::new(self.x + x, self.y + y)
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct SerPolygon {
    verteces: Vec<SerVertex>,
    width: f64,
    height: f64,
}
impl SerPolygon {
    fn from_polygon(polygon: &Vec<Position<f64>>) -> SerPolygon {
        let mut min_x: f64 = 10e10;
        let mut min_y: f64 = 10e10;
        let mut max_x: f64 = -10e10;
        let mut max_y: f64 = -10e10;
        SerPolygon {
            verteces: {
                let mut verteces = Vec::<SerVertex>::new();
                for vertex in polygon {
                    if vertex.x < min_x {
                        min_x = vertex.x;
                    } else if vertex.x > max_x {
                        max_x = vertex.x;
                    }

                    if vertex.y < min_y {
                        min_y = vertex.y;
                    } else if vertex.y > max_y {
                        max_y = vertex.y;
                    }

                    verteces.push(SerVertex {
                        x: vertex.x,
                        y: vertex.y,
                    });
                }
                verteces
            },
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    fn normalize(&mut self) {
        let mut min_x = 10e10;
        let mut min_y = 10e10;

        for vertex in &self.verteces {
            if vertex.x < min_x {
                min_x = vertex.x;
            }
            if vertex.y < min_y {
                min_y = vertex.y;
            }
        }
        for i in 0..self.verteces.len() {
            self.verteces[i].x -= min_x;
            self.verteces[i].y -= min_y;
        }
    }

    fn translate(&mut self, x: f64, y: f64) {
        for i in 0..self.verteces.len() {
            self.verteces[i].x += x;
            self.verteces[i].y += y;
        }
    }

    fn to_polygon(&self) -> Polygon {
        self.to_polygon_translate(0.0, 0.0)
    }

    fn to_polygon_translate(&self, x: f64, y: f64) -> Polygon {
        let mut polygon = Polygon::new();
        polygon.vertices = Vec::<Position<f64>>::new();
        for vertex in &self.verteces {
            polygon.vertices.push(vertex.to_vertex_translate(x, y));
        }
        polygon
    }

    fn from_lev(level: &Level) -> Vec<SerPolygon> {
        let mut polygons = Vec::<SerPolygon>::new();
        for polygon in &level.polygons {
            if !polygon.grass {
                let mut sp = SerPolygon::from_polygon(&polygon.vertices);
                sp.normalize();
                polygons.push(sp);
            }
        }
        polygons
    }
}

#[derive(Serialize, Deserialize)]
struct SerPolygonOwner {
    file_name: String,
    polygon: SerPolygon,
}
impl SerPolygonOwner {
    fn from_serlevel(serlevel: &SerLevel) -> Vec<SerPolygonOwner> {
        let mut spo_vec = Vec::<SerPolygonOwner>::new();
        for polygon in &serlevel.polygons {
            spo_vec.push(SerPolygonOwner {
                file_name: serlevel.file_name.to_owned(),
                polygon: polygon.clone(),
            });
        }
        spo_vec
    }
}

#[derive(Serialize, Deserialize)]
struct SerLevel {
    file_name: String,
    level_title: String,
    polygons: Vec<SerPolygon>,
}
impl SerLevel {
    fn from_file(path: &str) -> Option<SerLevel> {
        let load = Level::load(path);

        match &load {
            Ok(level) => Some(SerLevel {
                file_name: Path::new(&path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                level_title: level.title.to_owned(),
                polygons: SerPolygon::from_lev(&level),
            }),
            Err(_) => None,
        }
    }
}

struct GeneratedLevel {
    level: Level,
    source_levels: Vec<String>,
}
impl GeneratedLevel {
    fn new() -> GeneratedLevel {
        GeneratedLevel {
            level: {
                let mut l = Level::new();
                l.polygons.pop();
                l
            },
            source_levels: Vec::new(),
        }
    }

    fn generate(db: &Db) -> GeneratedLevel {
        let mut genlev = GeneratedLevel::new();
        let n_polygons: usize = Self::rand_number_of_polygons();
        let mut x = 0.0;

        // Add one big polygon to the level
        let p = &db.polygons[Self::rand_big_polygon(db.polygons.len())];
        genlev.push_polygon(&p, x);

        // Add a bunch of polygons to the level
        for _ in 1..n_polygons {
            let p = &db.polygons[Self::rand_polygon(db.polygons.len())];
            genlev.push_polygon(&p, x);
            x += p.polygon.width;
        }

        genlev
    }

    fn push_polygon(&mut self, poly: &SerPolygonOwner, x: f64) {
        self.level
            .polygons
            .push(poly.polygon.to_polygon_translate(x, 0.0));
        self.source_levels.push(poly.file_name.to_owned());
    }

    fn write(&mut self, to: &str) -> std::result::Result<(), ()> {
        print!("{:14}", to);

        if let Err(_) = Level::save(&mut self.level, to, Top10Save::No) {
            return Err(());
        }

        let mut metafile = to.to_owned();
        metafile.push_str(".meta.json");

        println!(" {:22}", metafile);

        let f = File::create(metafile).unwrap();
        let w = BufWriter::new(f);
        if let Err(_) = serde_json::ser::to_writer(w, &self.source_levels) {
            return Err(());
        }

        Ok(())
    }

    fn rand_number_of_polygons() -> usize {
        let mut rng = thread_rng();
        let m = rng.sample(Normal::new(7.0, 4.0).unwrap());
        if m < 1.0 {
            return 1;
        }
        m as usize
    }

    fn rand_big_polygon(max: usize) -> usize {
        let mut rng = thread_rng();
        let max = max as f64;
        let dist = Normal::new(max * (5.0 / 6.0), max / 6.0).unwrap();
        loop {
            let sample = rng.sample(dist);
            if 0.0 <= sample && sample < max {
                return sample as usize;
            }
        }
    }

    fn rand_polygon(max: usize) -> usize {
        let mut rng = thread_rng();
        let max = max as f64;
        let dist = Normal::new(max / 2.0, max / 6.0).unwrap();
        loop {
            let sample = rng.sample(dist);
            if 0.0 <= sample && sample < max {
                return sample as usize;
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Db {
    polygons: Vec<SerPolygonOwner>,
    levels: Vec<String>,
    tag: Option<String>,
}
impl Db {
    fn new() -> Db {
        Db {
            polygons: Vec::new(),
            levels: Vec::new(),
            tag: None,
        }
    }

    fn load_database(db_path: &str) -> Db {
        let f = File::open(db_path).unwrap();
        let r = BufReader::new(f);
        bincode::deserialize_from::<BufReader<File>, Db>(r).unwrap()
    }

    fn write_database(&self, db_path: &str) -> std::result::Result<(), ()> {
        let f = File::create(db_path).unwrap();
        let w = BufWriter::new(f);
        match bincode::serialize_into::<BufWriter<File>, Db>(w, self) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn from_directory(dir_path: &str) -> Db {
        let readdir: ReadDir = read_dir(dir_path).unwrap();
        let mut polygons: Vec<SerPolygonOwner> = Vec::new();
        let mut levels: Vec<String> = Vec::new();
        let mut failed_levs: Vec<String> = Vec::new();
        let mut n_failed = 0;
        let mut n_polygons = 0;

        println!("\nLOADED      FAILED      POLYGONS");

        for (i, de) in readdir.enumerate() {
            let entry = de.unwrap();
            let path = entry.path();
            if match path.extension() {
                Some(ext) => ext == "lev",
                None => false,
            } {
                let full_path = path.to_str().unwrap();
                let file_name = path.file_name().unwrap().to_str().unwrap();

                if let Some(level) = SerLevel::from_file(full_path) {
                    let mut new_polygons = SerPolygonOwner::from_serlevel(&level);
                    polygons.append(&mut new_polygons);
                    levels.push(file_name.to_owned());
                    n_polygons = polygons.len();
                } else {
                    failed_levs.push(file_name.to_owned());
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

        Db {
            polygons,
            levels,
            tag: None,
        }
    }

    fn combine(&mut self, other: &mut Db) {
        let mut new_levels: HashSet<String> = HashSet::new();

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
            self.levels.push(new_level);
        }
    }
}

fn main() {
    let matches = App::new("lev-ztarter")
        .version(crate_version!())
        .author(crate_authors!())
        .args_from_usage(
            "
            [from-directory]      --from-directory      [dir]       'load levs from a directory'
            [from-database]       --from-database       [file]      'load levs from a database'
            [from-databases]      --from-databases      [files]     'load levs from multiple databases'
            [to-database]         --to-database         [out]       'write database'
            [tag-database]        --tag-database        [tag]       'tag the database that is being written'
            [generate]            --generate                        'generate levels using loaded database(s)'
            [generate-directory]  --generate-directory  [dir]       'generate levels to this directory'
            [level-name]          --level-name          [name]      'name of the level (e.g. for abc123 put abc)'
            [level-name-pad]      --level-name-pad      [num]       'number of padding zeros (e.g. for abc001 put 3)'
            [level-number-offset] --level-number-offset [num]       'start numbering levels at this number'
            [level-amount]        --level-amount        [num]       'amount of levels to generate'
            ",
        )
        .group(ArgGroup::with_name("input").args(&[
            "from-directory",
            "from-database",
            "from-databases",
        ]))
        .get_matches();

    let mut database_paths = Vec::<&str>::new();
    let mut database = Db::new();

    // Load database(s) or directory into a database

    if matches.is_present("from-database") {
        let dbp = matches.value_of("from-database").unwrap();
        database_paths.push(dbp);
    } else if matches.is_present("from-databases") {
        for dbp in matches.value_of("from-databases").unwrap().split(",") {
            database_paths.push(dbp);
        }
    } else if matches.is_present("from-directory") {
        let dirp = matches.value_of("from-directory").unwrap();
        println!("Loading all levels from {}.", dirp);
        let mut db = Db::from_directory(dirp);
        database.combine(&mut db);
    }

    for dbp in database_paths {
        print!("Loading database from {}...", dbp);
        let mut db = Db::load_database(dbp);
        if let Some(s) = &db.tag {
            println!(" loaded {}", s);
        } else {
            println!(" loaded untagged database")
        }
        database.combine(&mut db);
    }

    // Sort database by polygon bounds areas

    if database.polygons.len() > 0 {
        println!("Sorting the database.");
        database.polygons.sort_by(|a, b| {
            (a.polygon.width * a.polygon.height)
                .partial_cmp(&(b.polygon.width * b.polygon.height))
                .unwrap()
        });
    }

    if let Some(tag) = matches.value_of("database-tag") {
        database.tag = Some(tag.to_owned());
    }

    // Write database to file

    if matches.is_present("to-database") {
        let dbp = matches.value_of("to-database").unwrap();
        println!("Writing database to {}.", dbp);
        Db::write_database(&database, dbp).unwrap();
    }

    // Generate levels

    if database.polygons.len() > 0 {
        if matches.is_present("generate") {
            let generate_directory = match matches.value_of("generate-directory") {
                Some(x) => x,
                None => "./",
            };

            let level_name = match matches.value_of("level-name") {
                Some(x) => x,
                None => "L",
            };

            let level_name_pad: usize = {
                let default = 8 - level_name.len();
                match matches.value_of("level-name-pad") {
                    Some(x) => match x.parse::<usize>() {
                        Ok(x) => x,
                        Err(_) => {
                            eprintln!(
                                "Failed to parse level-name-pad ({}), defaulting to {}",
                                x, default
                            );
                            default
                        }
                    },
                    None => default,
                }
            };

            let level_number_offset = match matches.value_of("level-number-offset") {
                Some(x) => match x.parse::<i32>() {
                    Ok(x) => x,
                    Err(_) => {
                        eprintln!(
                            "Failed to parse level-number-offset ({}), defaulting to 1",
                            x
                        );
                        1
                    }
                },
                None => 1,
            };

            let level_amount = match matches.value_of("level-amount") {
                Some(x) => match x.parse::<i32>() {
                    Ok(x) => x,
                    Err(_) => {
                        eprintln!("Failed to parse level-amount ({}), defaulting to 1", x);
                        0
                    }
                },
                None => 0,
            };

            println!("Generating levels:");
            for i in 0..level_amount {
                let fp = Path::new(generate_directory).join(Path::new(&format_level_name(
                    level_name,
                    level_name_pad,
                    i + level_number_offset,
                )));
                let mut generated_level = GeneratedLevel::generate(&database);
                generated_level.write(fp.to_str().unwrap()).unwrap();
            }
        }
    } else {
        eprintln!("There are no polygons in the database. Skipping level generation.");
    }

    println!("Done.");
}

fn format_level_name(pre: &str, pad: usize, num: i32) -> String {
    let mut s = String::new();
    let num = format!("{}", num);
    s.push_str(pre);
    for _ in 0..pad - num.len() {
        s.push('0');
    }
    s.push_str(&num);
    s.push_str(".lev");
    s
}
