mod db;
mod gen;
mod lfn;
mod se;

use crate::{db::Db, gen::GeneratedLevel, lfn::LevelFileName};
use clap::{crate_authors, crate_version, App, ArgGroup};
use std::path::Path;

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
    // TODO ifs

    let mut database_paths = Vec::<&str>::new();
    let mut database = Db::new();

    // Load database(s) or directory into a database

    if let Some(dbp) = matches.value_of("from-database") {
        database_paths.push(dbp);
    } else if let Some(paths) = matches.value_of("from-databases") {
        for dbp in paths.split(",") {
            database_paths.push(dbp);
        }
    }
    if let Some(dirp) = matches.value_of("from-directory") {
        println!("Loading all levels from {}.", dirp);
        let mut db = Db::from_directory(dirp).unwrap();
        database.combine(&mut db);
    } else {
        for dbp in database_paths {
            print!("Loading database from {}...", dbp);
            if let Ok(mut db) = Db::load_database(dbp) {
                if let Some(s) = &db.tag {
                    println!(" loaded {}", s);
                } else {
                    println!(" loaded untagged database")
                }
                database.combine(&mut db);
            } else {
                continue;
            }
        }
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
                let fp = Path::new(generate_directory).join(Path::new(
                    &LevelFileName::fmt_level_name_string(
                        level_name,
                        level_name_pad,
                        i + level_number_offset,
                    ),
                ));
                let mut generated_level = GeneratedLevel::generate(&database);
                generated_level.write(fp.to_str().unwrap()).unwrap();
            }
        }
    } else {
        eprintln!("There are no polygons in the database. Skipping level generation.");
    }

    println!("Done.");
}
