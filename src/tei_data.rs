// src/tei_data.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeiDocument {
    pub metadata: Metadata,
    pub facsimile: Facsimile,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub author: String,
    pub editor: String,
    pub edition_type: String,
    pub language: String,
    pub country: Option<String>,
    pub settlement: Option<String>,
    pub institution: Option<String>,
    pub collection: Option<String>,
    pub siglum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Facsimile {
    pub surface_id: String,
    pub image_url: String,
    pub width: u32,
    pub height: u32,
    pub zones: HashMap<String, Zone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub zone_type: String,
    pub points: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub facs: String, // Reference to zone id
    pub content: Vec<TextNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TextNode {
    Text { content: String },
    Abbr { abbr: String, expan: String },
    Choice { sic: String, corr: String },
    Num { value: u32, text: String },
    PersName { name: String },
    PlaceName { name: String },
    RsType { rs_type: String, content: String },
    Note { content: String, note_id: String },
    Hi { rend: String, content: String },
}

impl TeiDocument {
    pub fn new() -> Self {
        Self {
            metadata: Metadata::default(),
            facsimile: Facsimile::default(),
            lines: Vec::new(),
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            editor: String::new(),
            edition_type: String::new(),
            language: String::new(),
            country: None,
            settlement: None,
            institution: None,
            collection: None,
            siglum: None,
        }
    }
}

impl Default for Facsimile {
    fn default() -> Self {
        Self {
            surface_id: String::new(),
            image_url: String::new(),
            width: 0,
            height: 0,
            zones: HashMap::new(),
        }
    }
}

impl Zone {
    pub fn parse_points(points_str: &str) -> Vec<(u32, u32)> {
        points_str
            .split_whitespace()
            .filter_map(|pair| {
                let coords: Vec<&str> = pair.split(',').collect();
                if coords.len() == 2 {
                    if let (Ok(x), Ok(y)) = (coords[0].parse::<u32>(), coords[1].parse::<u32>()) {
                        return Some((x, y));
                    }
                }
                None
            })
            .collect()
    }

    pub fn get_bounding_box(&self) -> (u32, u32, u32, u32) {
        if self.points.is_empty() {
            return (0, 0, 0, 0);
        }

        let mut min_x = u32::MAX;
        let mut min_y = u32::MAX;
        let mut max_x = 0;
        let mut max_y = 0;

        for (x, y) in &self.points {
            min_x = min_x.min(*x);
            min_y = min_y.min(*y);
            max_x = max_x.max(*x);
            max_y = max_y.max(*y);
        }

        (min_x, min_y, max_x, max_y)
    }
}
