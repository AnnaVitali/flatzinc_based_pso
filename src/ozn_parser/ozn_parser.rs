use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
/// An enum representing the mapping of an output variable, which can be either a scalar or an array, along with its source.
pub enum OznMapping {
    /// A scalar variable mapping, containing the source of the variable.
    Scalar { source: String },
    /// An array variable mapping, containing the source of the array.
    Array { source: String },
}

/// Default implementation for `OznMapping`, which defaults to a scalar with an empty source.
impl Default for OznMapping {
    fn default() -> Self {
        OznMapping::Scalar {
            source: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
/// A struct responsible for parsing .ozn files and mapping output variables to their sources.
pub struct OznParser {
    /// A hashmap that maps output variable names to their corresponding `OznMapping`, which indicates whether the variable is a scalar or an array and its source.
    output_variable_map: HashMap<String, OznMapping>,
}

/// Implementation of the `OznParser` struct, providing methods to create a new parser, parse .ozn files, and map variables to their sources.
impl OznParser {
    /// Creates a new instance of `OznParser` with an empty output variable map.
    pub fn new() -> Self {
        Self {
            output_variable_map: HashMap::new(),
        }
    }

    /// Parses the given .ozn file and populates the `output_variable_map` with the mappings of output variables to their sources.
    /// 
    /// # Arguments
    /// * `path` - A reference to the path of the .ozn file to be parsed.
    pub fn parse(&mut self, path: &Path) {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Error reading file {}: {}", path.display(), e));

        // Dynamic pre-allocation based on number of lines
        let estimated_lines = content.lines().count();
        self.output_variable_map.reserve(estimated_lines);

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
