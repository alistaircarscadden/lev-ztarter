use crate::lfn::LevelFileName;
use elma::{
    lev::{Level, Polygon},
    Position,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct SerVertex {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SerPolygon {
    pub verteces: Vec<SerVertex>,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize)]
pub struct SerPolygonOwner {
    pub file_name: LevelFileName,
    pub polygon: SerPolygon,
}

impl SerVertex {
    pub fn to_vertex_translate(&self, x: f64, y: f64) -> Position<f64> {
        Position::new(self.x + x, self.y + y)
    }
}

impl SerPolygon {
    pub fn from_polygon(polygon: &Vec<Position<f64>>) -> SerPolygon {
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

    pub fn normalize(&mut self) {
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

    pub fn to_polygon_translate(&self, x: f64, y: f64) -> Polygon {
        let mut polygon = Polygon::new();
        polygon.vertices = Vec::<Position<f64>>::new();
        for vertex in &self.verteces {
            polygon.vertices.push(vertex.to_vertex_translate(x, y));
        }
        polygon
    }
}

impl SerPolygonOwner {
    pub fn from_level_path(path: &Path) -> Result<Vec<SerPolygonOwner>, ()> {
        if let Ok(lev) = Level::load(path) {
            let mut polygons: Vec<SerPolygonOwner> = Vec::new();
            for polygon in lev.polygons {
                polygons.push(SerPolygonOwner {
                    file_name: LevelFileName::from(path.file_name().unwrap().to_str().unwrap())
                        .unwrap(),
                    polygon: {
                        let mut serp = SerPolygon::from_polygon(&polygon.vertices);
                        serp.normalize();
                        serp
                    },
                });
            }

            Ok(polygons)
        } else {
            Err(())
        }
    }
}
