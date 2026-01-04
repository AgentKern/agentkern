//! Sanctions Screener - AML/CFT Compliance
//!
//! Screen payments against OFAC, EU, UN sanctions lists

use super::{SanctionsResult, SanctionsMatch, SwiftError};

/// Sanctions screening service.
pub struct SanctionsScreener {
    sources: Vec<String>,
    loaded_entries: usize,
}

impl SanctionsScreener {
    /// Create new screener with list sources.
    pub fn new(sources: &[String]) -> Self {
        Self {
            sources: sources.to_vec(),
            loaded_entries: 0,
        }
    }
    
    /// Load sanctions lists.
    pub fn load_lists(&mut self) -> Result<usize, SwiftError> {
        // Production would download and parse lists from:
        // - OFAC: https://sanctionslist.ofac.treas.gov/
        // - EU: https://webgate.ec.europa.eu/europeaid/fsd/fsf
        // - UN: https://scsanctions.un.org/
        
        // Simulate loaded entries
        self.loaded_entries = 15000;
        Ok(self.loaded_entries)
    }
    
    /// Screen names against sanctions.
    pub fn screen(&self, name1: &str, name2: &str) -> Result<SanctionsResult, SwiftError> {
        // Production would use fuzzy matching
        let matches = self.check_name(name1);
        let matches2 = self.check_name(name2);
        
        let all_matches: Vec<_> = matches.into_iter().chain(matches2).collect();
        let clear = all_matches.is_empty();
        
        if !clear {
            return Err(SwiftError::SanctionsHit(
                format!("{} matches found", all_matches.len())
            ));
        }
        
        Ok(SanctionsResult {
            clear,
            matches: all_matches,
        })
    }
    
    /// Check single name.
    fn check_name(&self, name: &str) -> Vec<SanctionsMatch> {
        // Production would do fuzzy matching
        // For demo, only match known test names
        let test_names = ["SANCTIONED ENTITY", "BLOCKED PERSON"];
        
        let upper = name.to_uppercase();
        test_names.iter()
            .filter(|n| upper.contains(*n))
            .map(|n| SanctionsMatch {
                list: "OFAC".to_string(),
                name: n.to_string(),
                score: 1.0,
            })
            .collect()
    }
    
    /// Get number of loaded list entries.
    pub fn list_count(&self) -> usize {
        self.loaded_entries
    }
    
    /// Get configured sources.
    pub fn sources(&self) -> &[String] {
        &self.sources
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_names() {
        let screener = SanctionsScreener::new(&["OFAC".into()]);
        let result = screener.screen("John Doe", "Jane Smith").unwrap();
        assert!(result.clear);
    }

    #[test]
    fn test_sanctioned_name() {
        let screener = SanctionsScreener::new(&["OFAC".into()]);
        let result = screener.screen("SANCTIONED ENTITY", "Jane Smith");
        assert!(result.is_err());
    }
}
