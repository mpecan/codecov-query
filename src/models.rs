use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub count: Option<u64>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
    pub total_pages: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Totals {
    pub files: Option<u64>,
    pub lines: Option<u64>,
    pub hits: Option<u64>,
    pub misses: Option<u64>,
    pub partials: Option<u64>,
    pub coverage: Option<f64>,
    pub branches: Option<u64>,
    pub methods: Option<u64>,
    pub complexity: Option<f64>,
    pub complexity_total: Option<f64>,
    pub complexity_ratio: Option<f64>,
    pub diff: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub service: Option<String>,
    pub username: Option<String>,
    pub name: Option<String>,
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    pub name: Option<String>,
    pub private: Option<bool>,
    pub updatestamp: Option<String>,
    pub author: Option<Owner>,
    pub language: Option<String>,
    pub branch: Option<String>,
    pub active: Option<bool>,
    pub activated: Option<bool>,
    pub totals: Option<Totals>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub commitid: Option<String>,
    pub message: Option<String>,
    pub timestamp: Option<String>,
    pub ci_passed: Option<bool>,
    pub author: Option<CommitAuthor>,
    pub branch: Option<String>,
    pub totals: Option<Totals>,
    pub state: Option<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub service: Option<String>,
    pub username: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    pub name: Option<String>,
    pub updatestamp: Option<String>,
    pub head: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pull {
    pub pullid: Option<u64>,
    pub title: Option<String>,
    pub state: Option<String>,
    pub updatestamp: Option<String>,
    pub author: Option<serde_json::Value>,
    pub base: Option<serde_json::Value>,
    pub head: Option<serde_json::Value>,
    pub ci_passed: Option<bool>,
    pub comparedto: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonFileName {
    pub base: Option<String>,
    pub head: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStats {
    pub added: Option<u64>,
    pub removed: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileTotals {
    pub base: Option<Totals>,
    pub head: Option<Totals>,
    pub patch: Option<Totals>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonFile {
    pub name: Option<ComparisonFileName>,
    pub has_diff: Option<bool>,
    pub stats: Option<FileStats>,
    pub totals: Option<FileTotals>,
    pub change_summary: Option<serde_json::Value>,
    pub lines: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comparison {
    pub base_commit: Option<String>,
    pub head_commit: Option<String>,
    pub totals: Option<TotalsComparison>,
    pub commit_uploads: Option<serde_json::Value>,
    pub diff: Option<serde_json::Value>,
    pub files: Option<Vec<ComparisonFile>>,
    pub untracked: Option<serde_json::Value>,
}

impl Comparison {
    pub fn into_summary(mut self) -> Self {
        if let Some(files) = &mut self.files {
            files.retain(|f| f.has_diff == Some(true));
            for file in files.iter_mut() {
                file.lines = None;
                file.change_summary = None;
            }
        }
        self.commit_uploads = None;
        self.diff = None;
        self.untracked = None;
        self
    }
}

#[derive(Debug, Serialize)]
pub struct PrSummary {
    pub pullid: u64,
    pub title: Option<String>,
    pub state: Option<String>,
    pub ci_passed: Option<bool>,
    pub base_coverage: Option<f64>,
    pub head_coverage: Option<f64>,
    pub coverage_delta: Option<f64>,
    pub patch_coverage: Option<f64>,
    pub patch_hits: Option<u64>,
    pub patch_lines: Option<u64>,
    pub status: String,
    pub changed_files: Vec<ChangedFileSummary>,
}

#[derive(Debug, Serialize)]
pub struct ChangedFileSummary {
    pub path: String,
    pub patch_coverage: Option<f64>,
    pub patch_hits: Option<u64>,
    pub patch_lines: Option<u64>,
    pub patch_misses: Option<u64>,
    pub base_coverage: Option<f64>,
    pub head_coverage: Option<f64>,
    pub status: String,
    pub uncovered_lines: Vec<u64>,
}

/// Extract uncovered (missed) line numbers from comparison file lines data.
///
/// The Codecov compare API returns lines as an array of objects:
/// ```json
/// {"number": {"head": 42}, "coverage": {"head": 0}, "added": true, ...}
/// ```
/// When `added_only` is true, only lines with `"added": true` are considered
/// (i.e. new patch lines). A line is "missed" when `coverage.head` is `0`.
/// Lines with `null` head coverage have no data and are skipped.
///
/// Also handles the legacy array-of-arrays format for robustness.
pub fn extract_uncovered_lines(lines: &serde_json::Value, added_only: bool) -> Vec<u64> {
    let Some(arr) = lines.as_array() else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for entry in arr {
        if let Some(line_no) = extract_from_object(entry, added_only) {
            result.push(line_no);
        } else if let Some(line_no) = extract_from_array(entry) {
            result.push(line_no);
        }
    }
    result
}

/// Object format: `{"number": {"head": N}, "coverage": {"head": 0}, "added": bool}`
fn extract_from_object(entry: &serde_json::Value, added_only: bool) -> Option<u64> {
    let obj = entry.as_object()?;
    if added_only && !obj.get("added")?.as_bool().unwrap_or(false) {
        return None;
    }
    let line_no = obj.get("number")?.get("head")?.as_u64()?;
    let head_cov = obj.get("coverage")?.get("head")?;
    if is_miss(head_cov) && !head_cov.is_null() {
        Some(line_no)
    } else {
        None
    }
}

/// Array format: `[line_no, cov]` or `[line_no, base_cov, head_cov]`
fn extract_from_array(entry: &serde_json::Value) -> Option<u64> {
    let inner = entry.as_array()?;
    if inner.len() < 2 {
        return None;
    }
    let line_no = inner[0].as_u64()?;
    let cov_value = if inner.len() >= 3 {
        &inner[2]
    } else {
        &inner[1]
    };
    if is_miss(cov_value) {
        Some(line_no)
    } else {
        None
    }
}

fn is_miss(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Number(n) => n.as_f64().is_some_and(|v| v == 0.0),
        serde_json::Value::String(s) => s == "miss" || s == "0" || s.starts_with("0/"),
        serde_json::Value::Bool(b) => !b,
        _ => false,
    }
}

/// Format line numbers as compact ranges: `[1, 2, 3, 5, 7, 8]` → `"1-3, 5, 7-8"`
pub fn format_line_ranges(lines: &[u64]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut sorted: Vec<u64> = lines.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    let mut ranges: Vec<String> = Vec::new();
    let mut start = sorted[0];
    let mut end = start;

    for &line in &sorted[1..] {
        if line != end + 1 {
            ranges.push(format_range(start, end));
            start = line;
        }
        end = line;
    }
    ranges.push(format_range(start, end));
    ranges.join(", ")
}

fn format_range(start: u64, end: u64) -> String {
    if start == end {
        start.to_string()
    } else {
        format!("{start}-{end}")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotalsComparison {
    pub base: Option<Totals>,
    pub head: Option<Totals>,
    pub patch: Option<Totals>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReport {
    pub name: Option<String>,
    pub totals: Option<Totals>,
    pub line_coverage: Option<serde_json::Value>,
    pub commit_sha: Option<String>,
    pub commit_file_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Flag {
    pub flag_name: Option<String>,
    pub coverage: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    #[serde(alias = "component_id")]
    pub id: Option<String>,
    pub name: Option<String>,
    pub coverage: Option<f64>,
}

/// Wrapper for the totals endpoint which returns totals nested under a key.
#[derive(Debug, Serialize, Deserialize)]
pub struct TotalsResponse {
    pub totals: Option<Totals>,
    pub commit_sha: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_totals() {
        let json = r#"{
            "files": 42,
            "lines": 1000,
            "hits": 800,
            "misses": 150,
            "partials": 50,
            "coverage": 80.0,
            "branches": 100,
            "methods": 200,
            "complexity": 3.5,
            "complexity_total": 700.0,
            "complexity_ratio": 3.5,
            "diff": null
        }"#;
        let totals: Totals = serde_json::from_str(json).unwrap();
        assert_eq!(totals.files, Some(42));
        assert_eq!(totals.coverage, Some(80.0));
        assert!(totals.diff.is_none() || totals.diff.as_ref().unwrap().is_null());
    }

    #[test]
    fn deserialize_totals_with_nulls() {
        let json = r#"{
            "files": null,
            "lines": null,
            "hits": null,
            "misses": null,
            "partials": null,
            "coverage": null,
            "branches": null,
            "methods": null,
            "complexity": null,
            "complexity_total": null,
            "complexity_ratio": null,
            "diff": null
        }"#;
        let totals: Totals = serde_json::from_str(json).unwrap();
        assert!(totals.files.is_none());
        assert!(totals.coverage.is_none());
    }

    #[test]
    fn deserialize_repo() {
        let json = r#"{
            "name": "my-repo",
            "private": false,
            "updatestamp": "2024-01-01T00:00:00Z",
            "author": {
                "service": "github",
                "username": "owner",
                "name": "Owner Name",
                "service_id": "12345"
            },
            "language": "rust",
            "branch": "main",
            "active": true,
            "activated": true,
            "totals": {
                "files": 10,
                "lines": 500,
                "hits": 400,
                "misses": 80,
                "partials": 20,
                "coverage": 80.0,
                "branches": 50,
                "methods": 100,
                "complexity": null,
                "complexity_total": null,
                "complexity_ratio": null,
                "diff": null
            }
        }"#;
        let repo: Repo = serde_json::from_str(json).unwrap();
        assert_eq!(repo.name.as_deref(), Some("my-repo"));
        assert_eq!(repo.active, Some(true));
        assert_eq!(
            repo.author.as_ref().unwrap().username.as_deref(),
            Some("owner")
        );
        assert_eq!(repo.totals.as_ref().unwrap().coverage, Some(80.0));
    }

    #[test]
    fn deserialize_paginated_repos() {
        let json = r#"{
            "count": 2,
            "next": null,
            "previous": null,
            "results": [
                {
                    "name": "repo-a",
                    "private": false,
                    "updatestamp": null,
                    "author": null,
                    "language": null,
                    "branch": "main",
                    "active": true,
                    "activated": true,
                    "totals": null
                },
                {
                    "name": "repo-b",
                    "private": true,
                    "updatestamp": null,
                    "author": null,
                    "language": "python",
                    "branch": "master",
                    "active": false,
                    "activated": false,
                    "totals": null
                }
            ],
            "total_pages": 1
        }"#;
        let paginated: Paginated<Repo> = serde_json::from_str(json).unwrap();
        assert_eq!(paginated.count, Some(2));
        assert_eq!(paginated.results.len(), 2);
        assert_eq!(paginated.results[0].name.as_deref(), Some("repo-a"));
        assert_eq!(paginated.results[1].language.as_deref(), Some("python"));
    }

    #[test]
    fn deserialize_commit() {
        let json = r#"{
            "commitid": "abc123def456",
            "message": "fix: bug",
            "timestamp": "2024-01-01T12:00:00Z",
            "ci_passed": true,
            "author": {
                "service": "github",
                "username": "dev",
                "name": "Developer"
            },
            "branch": "main",
            "totals": null,
            "state": "complete",
            "parent": "parent123"
        }"#;
        let commit: Commit = serde_json::from_str(json).unwrap();
        assert_eq!(commit.commitid.as_deref(), Some("abc123def456"));
        assert_eq!(commit.ci_passed, Some(true));
        assert_eq!(commit.state.as_deref(), Some("complete"));
    }

    #[test]
    fn deserialize_pull() {
        let json = r#"{
            "pullid": 42,
            "title": "Add feature",
            "state": "open",
            "updatestamp": "2024-01-01T00:00:00Z",
            "author": null,
            "base": null,
            "head": null,
            "ci_passed": true,
            "comparedto": "abc123"
        }"#;
        let pull: Pull = serde_json::from_str(json).unwrap();
        assert_eq!(pull.pullid, Some(42));
        assert_eq!(pull.title.as_deref(), Some("Add feature"));
    }

    #[test]
    fn deserialize_comparison() {
        let json = r#"{
            "base_commit": "abc123",
            "head_commit": "def456",
            "totals": {
                "base": { "files": 10, "lines": 100, "hits": 80, "misses": 15, "partials": 5, "coverage": 80.0, "branches": null, "methods": null, "complexity": null, "complexity_total": null, "complexity_ratio": null, "diff": null },
                "head": { "files": 12, "lines": 120, "hits": 100, "misses": 15, "partials": 5, "coverage": 83.3, "branches": null, "methods": null, "complexity": null, "complexity_total": null, "complexity_ratio": null, "diff": null },
                "patch": null
            },
            "commit_uploads": null,
            "diff": null,
            "files": null,
            "untracked": null
        }"#;
        let cmp: Comparison = serde_json::from_str(json).unwrap();
        assert_eq!(cmp.base_commit.as_deref(), Some("abc123"));
        let totals = cmp.totals.unwrap();
        assert_eq!(totals.base.as_ref().unwrap().coverage, Some(80.0));
        assert_eq!(totals.head.as_ref().unwrap().coverage, Some(83.3));
    }

    #[test]
    fn deserialize_comparison_with_files() {
        let json = r#"{
            "base_commit": "abc123",
            "head_commit": "def456",
            "totals": { "base": null, "head": null, "patch": null },
            "commit_uploads": null,
            "diff": null,
            "files": [
                {
                    "name": { "base": "src/lib.rs", "head": "src/lib.rs" },
                    "has_diff": true,
                    "stats": { "added": 10, "removed": 2 },
                    "totals": {
                        "base": { "files": 1, "lines": 50, "hits": 45, "misses": 5, "partials": 0, "coverage": 90.0, "branches": null, "methods": null, "complexity": null, "complexity_total": null, "complexity_ratio": null, "diff": null },
                        "head": { "files": 1, "lines": 58, "hits": 50, "misses": 8, "partials": 0, "coverage": 86.2, "branches": null, "methods": null, "complexity": null, "complexity_total": null, "complexity_ratio": null, "diff": null },
                        "patch": { "files": 1, "lines": 10, "hits": 8, "misses": 2, "partials": 0, "coverage": 80.0, "branches": null, "methods": null, "complexity": null, "complexity_total": null, "complexity_ratio": null, "diff": null }
                    },
                    "change_summary": null,
                    "lines": [[1, "hit"], [2, "miss"]]
                },
                {
                    "name": { "base": "src/utils.rs", "head": "src/utils.rs" },
                    "has_diff": false,
                    "stats": null,
                    "totals": null,
                    "change_summary": null,
                    "lines": null
                }
            ],
            "untracked": null
        }"#;
        let cmp: Comparison = serde_json::from_str(json).unwrap();
        let files = cmp.files.as_ref().unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].has_diff, Some(true));
        assert_eq!(files[1].has_diff, Some(false));
        assert_eq!(
            files[0].name.as_ref().unwrap().head.as_deref(),
            Some("src/lib.rs")
        );
    }

    #[test]
    fn comparison_into_summary_filters_files() {
        let json = r#"{
            "base_commit": "abc",
            "head_commit": "def",
            "totals": { "base": null, "head": null, "patch": null },
            "commit_uploads": { "some": "data" },
            "diff": { "some": "diff" },
            "files": [
                { "name": { "base": "a.rs", "head": "a.rs" }, "has_diff": true, "stats": null, "totals": null, "change_summary": { "some": "data" }, "lines": [[1, "hit"]] },
                { "name": { "base": "b.rs", "head": "b.rs" }, "has_diff": false, "stats": null, "totals": null, "change_summary": null, "lines": null }
            ],
            "untracked": { "some": "data" }
        }"#;
        let cmp: Comparison = serde_json::from_str(json).unwrap();
        let summary = cmp.into_summary();
        assert!(summary.commit_uploads.is_none());
        assert!(summary.diff.is_none());
        assert!(summary.untracked.is_none());
        let files = summary.files.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].name.as_ref().unwrap().head.as_deref(),
            Some("a.rs")
        );
        assert!(files[0].lines.is_none());
        assert!(files[0].change_summary.is_none());
    }

    #[test]
    fn deserialize_flag() {
        let json = r#"{"flag_name": "unit", "coverage": 85.5}"#;
        let flag: Flag = serde_json::from_str(json).unwrap();
        assert_eq!(flag.flag_name.as_deref(), Some("unit"));
        assert_eq!(flag.coverage, Some(85.5));
    }

    #[test]
    fn deserialize_component() {
        let json = r#"{"component_id": "comp-1", "name": "Backend", "coverage": 92.1}"#;
        let comp: Component = serde_json::from_str(json).unwrap();
        assert_eq!(comp.id.as_deref(), Some("comp-1"));
        assert_eq!(comp.coverage, Some(92.1));
    }

    #[test]
    fn deserialize_file_report() {
        let json = r#"{
            "name": "src/main.rs",
            "totals": { "files": 1, "lines": 50, "hits": 45, "misses": 5, "partials": 0, "coverage": 90.0, "branches": null, "methods": null, "complexity": null, "complexity_total": null, "complexity_ratio": null, "diff": null },
            "line_coverage": [[1, 1], [2, 0], [3, 1]],
            "commit_sha": "abc123",
            "commit_file_url": "https://github.com/owner/repo/blob/abc123/src/main.rs"
        }"#;
        let report: FileReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.name.as_deref(), Some("src/main.rs"));
        assert_eq!(report.totals.as_ref().unwrap().coverage, Some(90.0));
    }

    #[test]
    fn deserialize_totals_response() {
        let json = r#"{
            "totals": { "files": 10, "lines": 100, "hits": 80, "misses": 15, "partials": 5, "coverage": 80.0, "branches": null, "methods": null, "complexity": null, "complexity_total": null, "complexity_ratio": null, "diff": null },
            "commit_sha": "abc123"
        }"#;
        let resp: TotalsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.commit_sha.as_deref(), Some("abc123"));
        assert_eq!(resp.totals.as_ref().unwrap().coverage, Some(80.0));
    }

    #[test]
    fn deserialize_branch() {
        let json = r#"{
            "name": "feature/test",
            "updatestamp": "2024-06-01T10:00:00Z",
            "head": "abc123"
        }"#;
        let branch: Branch = serde_json::from_str(json).unwrap();
        assert_eq!(branch.name.as_deref(), Some("feature/test"));
    }

    #[test]
    fn deserialize_repo_missing_fields() {
        let json = r#"{"name": "minimal"}"#;
        let repo: Repo = serde_json::from_str(json).unwrap();
        assert_eq!(repo.name.as_deref(), Some("minimal"));
        assert!(repo.active.is_none());
        assert!(repo.totals.is_none());
    }

    #[test]
    fn extract_uncovered_lines_object_format_all() {
        // Real Codecov API format, added_only=false
        let lines: serde_json::Value = serde_json::from_str(
            r#"[
                {"number": {"base": 10, "head": 10}, "coverage": {"base": null, "head": 1}, "added": false},
                {"number": {"base": 11, "head": 11}, "coverage": {"base": null, "head": 0}, "added": false},
                {"number": {"base": null, "head": 12}, "coverage": {"base": null, "head": 0}, "added": true},
                {"number": {"base": 13, "head": 13}, "coverage": {"base": null, "head": null}, "added": true},
                {"number": {"base": 14, "head": 14}, "coverage": {"base": 0, "head": 1}, "added": true}
            ]"#,
        )
        .unwrap();
        let uncovered = extract_uncovered_lines(&lines, false);
        assert_eq!(uncovered, vec![11, 12]);
    }

    #[test]
    fn extract_uncovered_lines_object_format_added_only() {
        let lines: serde_json::Value = serde_json::from_str(
            r#"[
                {"number": {"base": 10, "head": 10}, "coverage": {"base": null, "head": 0}, "added": false},
                {"number": {"base": null, "head": 11}, "coverage": {"base": null, "head": 0}, "added": true},
                {"number": {"base": null, "head": 12}, "coverage": {"base": null, "head": 1}, "added": true}
            ]"#,
        )
        .unwrap();
        // Only added lines with head=0
        let uncovered = extract_uncovered_lines(&lines, true);
        assert_eq!(uncovered, vec![11]);
    }

    #[test]
    fn extract_uncovered_lines_array_format() {
        // Legacy/alternative array format
        let lines: serde_json::Value =
            serde_json::from_str(r#"[[1, "hit"], [2, "miss"], [3, 1], [4, 0]]"#).unwrap();
        let uncovered = extract_uncovered_lines(&lines, false);
        assert_eq!(uncovered, vec![2, 4]);
    }

    #[test]
    fn extract_uncovered_lines_array_format_with_added_only() {
        // Array format has no "added" info, so all misses are included as fallback
        let lines: serde_json::Value =
            serde_json::from_str(r#"[[1, 1], [2, 0], [3, "miss"]]"#).unwrap();
        let uncovered = extract_uncovered_lines(&lines, true);
        assert_eq!(uncovered, vec![2, 3]);
    }

    #[test]
    fn extract_uncovered_lines_three_element_format() {
        let lines: serde_json::Value =
            serde_json::from_str(r#"[[10, 1, 1], [11, 1, 0], [12, null, "miss"], [13, 0, 1]]"#)
                .unwrap();
        let uncovered = extract_uncovered_lines(&lines, false);
        assert_eq!(uncovered, vec![11, 12]);
    }

    #[test]
    fn extract_uncovered_lines_empty_and_invalid() {
        assert!(extract_uncovered_lines(&serde_json::Value::Null, false).is_empty());
        assert!(extract_uncovered_lines(&serde_json::json!([]), false).is_empty());
        assert!(
            extract_uncovered_lines(&serde_json::json!([["not_a_number", 1]]), false).is_empty()
        );
    }

    #[test]
    fn extract_uncovered_lines_partial_fraction() {
        let lines: serde_json::Value =
            serde_json::from_str(r#"[[1, "0/2"], [2, "1/2"], [3, "2/2"]]"#).unwrap();
        let uncovered = extract_uncovered_lines(&lines, false);
        assert_eq!(uncovered, vec![1]);
    }

    #[test]
    fn format_line_ranges_consecutive() {
        assert_eq!(format_line_ranges(&[1, 2, 3, 5, 7, 8, 9]), "1-3, 5, 7-9");
    }

    #[test]
    fn format_line_ranges_single() {
        assert_eq!(format_line_ranges(&[42]), "42");
    }

    #[test]
    fn format_line_ranges_empty() {
        assert_eq!(format_line_ranges(&[]), "");
    }

    #[test]
    fn format_line_ranges_unsorted_with_dupes() {
        assert_eq!(format_line_ranges(&[5, 3, 3, 1, 2, 4]), "1-5");
    }
}
