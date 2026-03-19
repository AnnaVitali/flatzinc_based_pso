use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct OznParser {
    output_variable_map: HashMap<String, OznMapping>,
}

impl OznParser {
    pub fn new() -> Self {
        Self {
            output_variable_map: HashMap::new(),
        }
    }

    pub fn parse(&mut self, path: &Path) {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Error reading file {}: {}", path.display(), e));

        // Dynamic pre-allocation based on number of lines
        let estimated_lines = content.lines().count();
        self.output_variable_map
            .reserve(estimated_lines);

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with("output") {
                continue;
            }

            if line.starts_with("array") {
                self.parse_array(line);
            } else {
                self.parse_scalar(line);
            }
        }
    }

    #[inline(always)]
    fn parse_array(&mut self, line: &str) {
        let colon_pos = match line.find(':') {
            Some(p) => p,
            None => return,
        };

        let after_colon = line[colon_pos + 1..].trim();

        let (name, source) = if let Some(eq_pos) = after_colon.find('=') {
            let name = after_colon[..eq_pos].trim();
            let rhs = after_colon[eq_pos + 1..].trim();

            let source = rhs
                .rsplit(',')
                .next()
                .and_then(|s| s.strip_suffix(");"))
                .unwrap_or(name)
                .trim();

            (name, source)
        } else {
            let name = after_colon.trim_end_matches(';').trim();
            (name, name)
        };

        self.output_variable_map.insert(
            name.to_owned(),
            OznMapping::Array {
                source: source.to_owned(),
            },
        );
    }

    #[inline(always)]
    fn parse_scalar(&mut self, line: &str) {
        let colon_pos = match line.find(':') {
            Some(p) => p,
            None => return,
        };

        let after_colon = line[colon_pos + 1..].trim();

        let (name, source) = if let Some(eq_pos) = after_colon.find('=') {
            let name = after_colon[..eq_pos].trim();
            let source = after_colon[eq_pos + 1..]
                .trim()
                .trim_end_matches(';')
                .trim();
            (name, source)
        } else {
            let name = after_colon.trim_end_matches(';').trim();
            (name, name)
        };

        self.output_variable_map.insert(
            name.to_owned(),
            OznMapping::Scalar {
                source: source.to_owned(),
            },
        );
    }

    #[inline(always)]
    pub fn map_variable(&self, name: &str) -> &OznMapping {
        self.output_variable_map
            .get(name)
            .unwrap_or_else(|| panic!("Variable {} not found in model", name))
    }
}

#[derive(Debug, Clone)]
pub enum OznMapping {
    Scalar { source: String },
    Array { source: String },
}

impl Default for OznMapping {
    fn default() -> Self {
        OznMapping::Scalar {
            source: String::new(),
        }
    }
}