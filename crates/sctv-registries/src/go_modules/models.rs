//! Go module proxy response models.

use serde::Deserialize;

/// Response from /{module}/@v/{version}.info
#[derive(Debug, Clone, Deserialize)]
pub struct VersionInfo {
    /// The canonical version string.
    #[serde(rename = "Version")]
    pub version: String,

    /// The commit time in RFC3339 format.
    #[serde(rename = "Time")]
    pub time: String,

    /// Origin information (optional).
    #[serde(rename = "Origin")]
    pub origin: Option<Origin>,
}

/// Origin information for a version.
#[derive(Debug, Clone, Deserialize)]
pub struct Origin {
    /// VCS type (e.g., "git").
    #[serde(rename = "VCS")]
    pub vcs: Option<String>,

    /// URL to the VCS repository.
    #[serde(rename = "URL")]
    pub url: Option<String>,

    /// The subdir within the repository.
    #[serde(rename = "Subdir")]
    pub subdir: Option<String>,

    /// Commit hash.
    #[serde(rename = "Hash")]
    pub hash: Option<String>,

    /// Reference (tag/branch).
    #[serde(rename = "Ref")]
    pub reference: Option<String>,
}

/// Parsed go.mod file.
#[derive(Debug, Clone, Default)]
pub struct GoMod {
    /// The module path.
    pub module: String,

    /// Go version requirement.
    pub go_version: Option<String>,

    /// Required dependencies.
    pub require: Vec<GoRequire>,

    /// Replacements.
    pub replace: Vec<GoReplace>,

    /// Exclusions.
    pub exclude: Vec<GoExclude>,

    /// Retractions (deprecated versions).
    pub retract: Vec<GoRetract>,
}

/// A require directive from go.mod.
#[derive(Debug, Clone)]
pub struct GoRequire {
    pub path: String,
    pub version: String,
    pub indirect: bool,
}

/// A replace directive from go.mod.
#[derive(Debug, Clone)]
pub struct GoReplace {
    pub old_path: String,
    pub old_version: Option<String>,
    pub new_path: String,
    pub new_version: Option<String>,
}

/// An exclude directive from go.mod.
#[derive(Debug, Clone)]
pub struct GoExclude {
    pub path: String,
    pub version: String,
}

/// A retract directive from go.mod.
#[derive(Debug, Clone)]
pub struct GoRetract {
    pub low: String,
    pub high: Option<String>,
    pub rationale: Option<String>,
}

impl GoMod {
    /// Parses a go.mod file content.
    #[must_use]
    pub fn parse(content: &str) -> Self {
        let mut result = Self::default();

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") {
                i += 1;
                continue;
            }

            // Module declaration
            if let Some(rest) = line.strip_prefix("module ") {
                result.module = rest.trim().trim_matches('"').to_string();
            }
            // Go version
            else if let Some(rest) = line.strip_prefix("go ") {
                result.go_version = Some(rest.trim().to_string());
            }
            // Require block
            else if line == "require (" {
                i += 1;
                while i < lines.len() && !lines[i].trim().starts_with(')') {
                    if let Some(req) = Self::parse_require_line(lines[i]) {
                        result.require.push(req);
                    }
                    i += 1;
                }
            }
            // Single require
            else if let Some(rest) = line.strip_prefix("require ") {
                if let Some(req) = Self::parse_require_line(rest) {
                    result.require.push(req);
                }
            }
            // Replace block
            else if line == "replace (" {
                i += 1;
                while i < lines.len() && !lines[i].trim().starts_with(')') {
                    if let Some(rep) = Self::parse_replace_line(lines[i]) {
                        result.replace.push(rep);
                    }
                    i += 1;
                }
            }
            // Single replace
            else if let Some(rest) = line.strip_prefix("replace ") {
                if let Some(rep) = Self::parse_replace_line(rest) {
                    result.replace.push(rep);
                }
            }
            // Retract block
            else if line == "retract (" {
                i += 1;
                while i < lines.len() && !lines[i].trim().starts_with(')') {
                    if let Some(ret) = Self::parse_retract_line(lines[i]) {
                        result.retract.push(ret);
                    }
                    i += 1;
                }
            }
            // Single retract
            else if let Some(rest) = line.strip_prefix("retract ") {
                if let Some(ret) = Self::parse_retract_line(rest) {
                    result.retract.push(ret);
                }
            }

            i += 1;
        }

        result
    }

    fn parse_require_line(line: &str) -> Option<GoRequire> {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            return None;
        }

        // Remove inline comment
        let line = line.split("//").next()?.trim();
        let indirect = line.contains("// indirect");

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Some(GoRequire {
                path: parts[0].to_string(),
                version: parts[1].to_string(),
                indirect,
            })
        } else {
            None
        }
    }

    fn parse_replace_line(line: &str) -> Option<GoReplace> {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            return None;
        }

        let parts: Vec<&str> = line.split("=>").collect();
        if parts.len() != 2 {
            return None;
        }

        let old_parts: Vec<&str> = parts[0].split_whitespace().collect();
        let new_parts: Vec<&str> = parts[1].split_whitespace().collect();

        if old_parts.is_empty() || new_parts.is_empty() {
            return None;
        }

        Some(GoReplace {
            old_path: old_parts[0].to_string(),
            old_version: old_parts.get(1).map(std::string::ToString::to_string),
            new_path: new_parts[0].to_string(),
            new_version: new_parts.get(1).map(std::string::ToString::to_string),
        })
    }

    fn parse_retract_line(line: &str) -> Option<GoRetract> {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            return None;
        }

        // Handle version range [v1.0.0, v1.0.5]
        if line.starts_with('[') {
            let bracket_end = line.find(']')?;
            let range = &line[1..bracket_end];
            let versions: Vec<&str> = range.split(',').map(str::trim).collect();
            if versions.len() == 2 {
                return Some(GoRetract {
                    low: versions[0].to_string(),
                    high: Some(versions[1].to_string()),
                    rationale: line
                        .get(bracket_end + 1..)
                        .and_then(|s| s.trim().strip_prefix("//").map(|r| r.trim().to_string())),
                });
            }
        }

        // Single version
        let parts: Vec<&str> = line.split("//").collect();
        let version = parts[0].trim();
        let rationale = parts.get(1).map(|s| s.trim().to_string());

        Some(GoRetract {
            low: version.to_string(),
            high: None,
            rationale,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_go_mod() {
        let content = r"
module github.com/example/foo

go 1.21

require (
    github.com/gin-gonic/gin v1.9.0
    github.com/stretchr/testify v1.8.0 // indirect
)

replace github.com/old/pkg => github.com/new/pkg v1.0.0
";

        let gomod = GoMod::parse(content);
        assert_eq!(gomod.module, "github.com/example/foo");
        assert_eq!(gomod.go_version, Some("1.21".to_string()));
        assert_eq!(gomod.require.len(), 2);
        assert_eq!(gomod.require[0].path, "github.com/gin-gonic/gin");
        assert_eq!(gomod.require[0].version, "v1.9.0");
        assert!(!gomod.require[0].indirect);
        assert_eq!(gomod.replace.len(), 1);
    }
}
