// src/project_config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pages: Vec<PageInfo>,
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageInfo {
    pub number: u32,
    pub label: String,
    pub has_diplomatic: bool,
    pub has_translation: bool,
    pub has_image: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub author: String,
    pub editor: String,
    pub collection: String,
    pub institution: String,
    pub country: String,
    pub language: String,
    pub date_range: String,
}

impl ProjectConfig {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            pages: Vec::new(),
            metadata: ProjectMetadata::default(),
        }
    }

    pub fn get_page(&self, page_num: u32) -> Option<&PageInfo> {
        self.pages.iter().find(|p| p.number == page_num)
    }

    pub fn get_page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn get_diplomatic_path(&self, page_num: u32) -> String {
        format!("projects/{}/p{}_dip.xml", self.id, page_num)
    }

    pub fn get_translation_path(&self, page_num: u32) -> String {
        format!("projects/{}/p{}_trad.xml", self.id, page_num)
    }

    pub fn get_image_path(&self, page_num: u32) -> String {
        format!("projects/{}/images/p{}.jpg", self.id, page_num)
    }
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            author: String::from("Anonymous"),
            editor: String::new(),
            collection: String::new(),
            institution: String::new(),
            country: String::new(),
            language: String::from("grc"),
            date_range: String::new(),
        }
    }
}

impl PageInfo {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            label: format!("Page {}", number),
            has_diplomatic: true,
            has_translation: true,
            has_image: true,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = label;
        self
    }

    pub fn with_diplomatic(mut self, has: bool) -> Self {
        self.has_diplomatic = has;
        self
    }

    pub fn with_translation(mut self, has: bool) -> Self {
        self.has_translation = has;
        self
    }

    pub fn with_image(mut self, has: bool) -> Self {
        self.has_image = has;
        self
    }
}

// Predefined project configurations
pub struct ProjectRegistry;

impl ProjectRegistry {
    pub fn get_all_projects() -> HashMap<String, ProjectConfig> {
        let mut projects = HashMap::new();

        // PGM XIII Project
        let mut pgm_xiii = ProjectConfig::new(
            "PGM-XIII".to_string(),
            "Papyri Graecae Magicae XIII".to_string(),
        );
        pgm_xiii.description = "Magical papyrus from the Greek Magical Papyri corpus, \
                                housed at the Rijksmuseum Amsterdam (AMS76)."
            .to_string();
        pgm_xiii.metadata = ProjectMetadata {
            author: "Anonymous".to_string(),
            editor: "Robert W. Daniel".to_string(),
            collection: "Papyri Graecae Magicae".to_string(),
            institution: "Rijksmuseum Amsterdam".to_string(),
            country: "Netherlands".to_string(),
            language: "Ancient Greek (grc)".to_string(),
            date_range: "1st c. BCE – 4th c. CE".to_string(),
        };

        // Add pages (adjust based on your actual pages)
        for i in 1..=25 {
            pgm_xiii.pages.push(
                PageInfo::new(i)
                    .with_label(format!("Folio {}", i))
                    .with_translation(i == 1), // Only page 1 has translation for now
            );
        }

        projects.insert(pgm_xiii.id.clone(), pgm_xiii);

        projects
    }

    pub fn get_project(id: &str) -> Option<ProjectConfig> {
        Self::get_all_projects().get(id).cloned()
    }

    pub fn get_project_ids() -> Vec<String> {
        Self::get_all_projects().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config() {
        let config = ProjectConfig::new("TEST".to_string(), "Test Project".to_string());
        assert_eq!(config.id, "TEST");
        assert_eq!(config.name, "Test Project");
    }

    #[test]
    fn test_page_info() {
        let page = PageInfo::new(1).with_label("First Page".to_string());
        assert_eq!(page.number, 1);
        assert_eq!(page.label, "First Page");
        assert!(page.has_diplomatic);
        assert!(page.has_translation);
        assert!(page.has_image);
    }

    #[test]
    fn test_project_registry() {
        let projects = ProjectRegistry::get_all_projects();
        assert!(!projects.is_empty());

        let pgm = ProjectRegistry::get_project("PGM-XIII");
        assert!(pgm.is_some());
        assert_eq!(pgm.unwrap().name, "Papyri Graecae Magicae XIII");
    }

    #[test]
    fn test_paths() {
        let config = ProjectConfig::new("TEST".to_string(), "Test".to_string());
        assert_eq!(config.get_diplomatic_path(1), "projects/TEST/p1_dip.xml");
        assert_eq!(config.get_translation_path(1), "projects/TEST/p1_trad.xml");
        assert_eq!(config.get_image_path(1), "projects/TEST/images/p1.jpg");
    }
}
