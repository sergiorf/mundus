use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub sources: Vec<Source>,
    pub first_milestone: FirstMilestone,
}

#[derive(Debug, Deserialize)]
pub struct Source {
    pub id: String,
    pub provider: String,
    pub local_cache_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct FirstMilestone {
    pub required_sources: Vec<String>,
    pub goal: String,
}

impl Manifest {
    pub fn sources_for_ids(&self, ids: &[String]) -> Result<Vec<&Source>, Box<dyn Error>> {
        let mut matches = Vec::with_capacity(ids.len());
        for id in ids {
            let source = self
                .sources
                .iter()
                .find(|source| source.id == *id)
                .ok_or_else(|| format!("manifest source `{id}` not found"))?;
            matches.push(source);
        }
        Ok(matches)
    }
}

impl Source {
    pub fn matches_file_name(&self, file_name: &str) -> bool {
        match self.id.as_str() {
            "natural_earth_110m_physical" => {
                file_name.contains("110m") && file_name.ends_with(".zip")
            }
            "natural_earth_50m_physical" => {
                file_name.contains("50m") && file_name.ends_with(".zip")
            }
            "natural_earth_10m_physical" => {
                file_name.contains("10m") && file_name.ends_with(".zip")
            }
            "nasa_blue_marble_next_generation" => {
                file_name.contains("blue_marble") && file_name.ends_with(".png")
            }
            "gebco_global_grid" => {
                file_name.contains("gebco")
                    && (file_name.ends_with(".tif")
                        || file_name.ends_with(".tiff")
                        || file_name.ends_with(".nc")
                        || file_name.ends_with(".zip"))
            }
            "usgs_srtm_1_arc_second_global" => {
                file_name.contains("srtm")
                    && (file_name.ends_with(".hgt")
                        || file_name.ends_with(".tif")
                        || file_name.ends_with(".zip"))
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Source;

    #[test]
    fn natural_earth_110m_matches_expected_zip() {
        let source = Source {
            id: "natural_earth_110m_physical".to_string(),
            provider: "Natural Earth".to_string(),
            local_cache_dir: "tools/earth_pipeline/cache/natural_earth".to_string(),
        };
        assert!(source.matches_file_name("110m_physical.zip"));
        assert!(!source.matches_file_name("50m_physical.zip"));
    }

    #[test]
    fn blue_marble_matches_expected_png() {
        let source = Source {
            id: "nasa_blue_marble_next_generation".to_string(),
            provider: "NASA".to_string(),
            local_cache_dir: "tools/earth_pipeline/cache/blue_marble".to_string(),
        };
        assert!(source.matches_file_name("blue_marble_ng_dec_2004_3600x1800.png"));
        assert!(!source.matches_file_name("not_blue_marble.jpg"));
    }
}
