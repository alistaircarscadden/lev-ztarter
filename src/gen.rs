use crate::{db::Db, lfn::LevelFileName};
use elma::lev::{Level, Top10Save};
use rand::{thread_rng, Rng};
use rand_distr::Normal;
use std::{fs::File, io::BufWriter};

pub struct GeneratedLevel {
    pub level: Level,
    pub source_levels: Vec<LevelFileName>,
}
impl GeneratedLevel {
    fn empty_level() -> Level {
        let mut l = Level::new();
        l.polygons.pop();
        l
    }

    pub fn generate(db: &Db) -> GeneratedLevel {
        let mut genlev = GeneratedLevel {
            level: Self::empty_level(),
            source_levels: Vec::new(),
        };

        let n_polygons: usize = Self::rand_number_of_polygons();
        let mut x = 0.0;

        // Add one big polygon to the level
        let p = &db.polygons[Self::rand_big_polygon(db.polygons.len())];
        genlev
            .level
            .polygons
            .push(p.polygon.to_polygon_translate(x, 0.0));
        genlev.source_levels.push(p.file_name);

        // Add a bunch of polygons to the level
        for _ in 1..n_polygons {
            let p = &db.polygons[Self::rand_polygon(db.polygons.len())];
            genlev
                .level
                .polygons
                .push(p.polygon.to_polygon_translate(x, 0.0));
            genlev.source_levels.push(p.file_name);
            x += p.polygon.width;
        }

        genlev
    }

    pub fn write(&mut self, to: &str) -> std::result::Result<(), ()> {
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
