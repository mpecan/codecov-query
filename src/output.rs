use crate::models::{
    self, Branch, ChangedFileSummary, Commit, Comparison, Component, FileReport, Flag, Paginated,
    PrSummary, Pull, Repo, Totals, TotalsResponse,
};
use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub trait TextFormat {
    fn print_text(&self);
}

impl TextFormat for Totals {
    fn print_text(&self) {
        println!("Coverage Totals:");
        if let Some(coverage) = self.coverage {
            println!("  Coverage:   {coverage:.2}%");
        }
        if let Some(files) = self.files {
            println!("  Files:      {files}");
        }
        if let Some(lines) = self.lines {
            println!("  Lines:      {lines}");
        }
        if let Some(hits) = self.hits {
            println!("  Hits:       {hits}");
        }
        if let Some(misses) = self.misses {
            println!("  Misses:     {misses}");
        }
        if let Some(partials) = self.partials {
            println!("  Partials:   {partials}");
        }
        if let Some(branches) = self.branches {
            println!("  Branches:   {branches}");
        }
        if let Some(methods) = self.methods {
            println!("  Methods:    {methods}");
        }
    }
}

impl TextFormat for TotalsResponse {
    fn print_text(&self) {
        if let Some(sha) = &self.commit_sha {
            println!("Commit: {sha}");
        }
        if let Some(totals) = &self.totals {
            totals.print_text();
        } else {
            println!("No coverage totals available.");
        }
    }
}

impl TextFormat for Repo {
    fn print_text(&self) {
        println!("Repository: {}", self.name.as_deref().unwrap_or("unknown"));
        if let Some(language) = &self.language {
            println!("  Language:   {language}");
        }
        if let Some(branch) = &self.branch {
            println!("  Branch:     {branch}");
        }
        if let Some(active) = self.active {
            println!("  Active:     {active}");
        }
        if let Some(private) = self.private {
            println!("  Private:    {private}");
        }
        if let Some(totals) = &self.totals
            && let Some(coverage) = totals.coverage
        {
            println!("  Coverage:   {coverage:.2}%");
        }
    }
}

impl<T: TextFormat> TextFormat for Paginated<T> {
    fn print_text(&self) {
        if let Some(count) = self.count {
            println!("Total: {count}");
        }
        if let Some(total_pages) = self.total_pages {
            println!("Pages: {total_pages}");
        }
        println!("---");
        for item in &self.results {
            item.print_text();
            println!();
        }
        if self.next.is_some() {
            println!("(more results available)");
        }
    }
}

impl TextFormat for Commit {
    fn print_text(&self) {
        if let Some(commitid) = &self.commitid {
            let short = if commitid.len() > 10 {
                &commitid[..10]
            } else {
                commitid
            };
            println!("Commit: {short}");
        }
        if let Some(message) = &self.message {
            let first_line = message.lines().next().unwrap_or(message);
            println!("  Message:    {first_line}");
        }
        if let Some(branch) = &self.branch {
            println!("  Branch:     {branch}");
        }
        if let Some(state) = &self.state {
            println!("  State:      {state}");
        }
        if let Some(ci_passed) = self.ci_passed {
            println!("  CI Passed:  {ci_passed}");
        }
        if let Some(timestamp) = &self.timestamp {
            println!("  Timestamp:  {timestamp}");
        }
        if let Some(totals) = &self.totals
            && let Some(coverage) = totals.coverage
        {
            println!("  Coverage:   {coverage:.2}%");
        }
    }
}

impl TextFormat for Branch {
    fn print_text(&self) {
        println!("Branch: {}", self.name.as_deref().unwrap_or("unknown"));
        if let Some(updatestamp) = &self.updatestamp {
            println!("  Updated:    {updatestamp}");
        }
    }
}

impl TextFormat for Pull {
    fn print_text(&self) {
        if let Some(pullid) = self.pullid {
            println!("PR #{pullid}");
        }
        if let Some(title) = &self.title {
            println!("  Title:      {title}");
        }
        if let Some(state) = &self.state {
            println!("  State:      {state}");
        }
        if let Some(ci_passed) = self.ci_passed {
            println!("  CI Passed:  {ci_passed}");
        }
        if let Some(updatestamp) = &self.updatestamp {
            println!("  Updated:    {updatestamp}");
        }
    }
}

impl TextFormat for Comparison {
    fn print_text(&self) {
        println!("Comparison:");
        if let Some(base) = &self.base_commit {
            println!("  Base:       {base}");
        }
        if let Some(head) = &self.head_commit {
            println!("  Head:       {head}");
        }
        if let Some(totals) = &self.totals {
            if let Some(base) = &totals.base
                && let Some(cov) = base.coverage
            {
                println!("  Base coverage:  {cov:.2}%");
            }
            if let Some(head) = &totals.head
                && let Some(cov) = head.coverage
            {
                println!("  Head coverage:  {cov:.2}%");
            }
            if let (Some(base), Some(head)) = (&totals.base, &totals.head)
                && let (Some(b), Some(h)) = (base.coverage, head.coverage)
            {
                let diff = h - b;
                let sign = if diff >= 0.0 { "+" } else { "" };
                println!("  Diff:           {sign}{diff:.2}%");
            }
            if let Some(patch) = &totals.patch
                && let Some(cov) = patch.coverage
            {
                println!("  Patch coverage: {cov:.2}%");
            }
        }
        if let Some(files) = &self.files {
            let changed: Vec<_> = files.iter().filter(|f| f.has_diff == Some(true)).collect();
            if !changed.is_empty() {
                println!("\nChanged files ({}):", changed.len());
                for file in &changed {
                    let path = file
                        .name
                        .as_ref()
                        .and_then(|n| n.head.as_deref().or(n.base.as_deref()))
                        .unwrap_or("unknown");
                    let patch_cov = file
                        .totals
                        .as_ref()
                        .and_then(|t| t.patch.as_ref())
                        .and_then(|p| p.coverage);
                    let patch_str =
                        patch_cov.map_or_else(|| "N/A".to_string(), |c| format!("{c:.2}%"));
                    println!("  {path}: patch {patch_str}");
                }
            }
        }
    }
}

fn format_coverage_pct(value: Option<f64>) -> String {
    value.map_or_else(|| "N/A".to_string(), |c| format!("{c:.2}%"))
}

impl TextFormat for PrSummary {
    fn print_text(&self) {
        let title = self.title.as_deref().unwrap_or("(no title)");
        println!("PR #{}: {title}", self.pullid);

        let state = self.state.as_deref().unwrap_or("unknown");
        let ci = self
            .ci_passed
            .map_or("unknown", |p| if p { "passed" } else { "failed" });
        println!("State: {state} | CI: {ci}");
        println!();

        match (self.base_coverage, self.head_coverage) {
            (Some(base), Some(head)) => {
                let delta = self.coverage_delta.unwrap_or(head - base);
                println!("Coverage: {base:.2}% -> {head:.2}% (delta {delta:+.2}%)");
            }
            (_, Some(head)) => println!("Coverage: {head:.2}%"),
            _ => println!("Coverage: N/A"),
        }

        if let Some(patch_cov) = self.patch_coverage {
            let hits = self.patch_hits.unwrap_or(0);
            let lines = self.patch_lines.unwrap_or(0);
            println!("Patch: {patch_cov:.2}% ({hits}/{lines} lines)");
        }

        println!("Status: [{}]", self.status);
        println!();

        if !self.changed_files.is_empty() {
            print_changed_files(&self.changed_files);
        }
    }
}

fn print_changed_files(files: &[ChangedFileSummary]) {
    println!("Changed files:");
    for file in files {
        let patch_str = format_coverage_pct(file.patch_coverage);
        let lines_str = file.patch_lines.map_or_else(String::new, |l| {
            let hits = file.patch_hits.unwrap_or(0);
            format!("{hits}/{l} lines")
        });
        println!(
            "  [{status}]  {path}  {patch_str}  {lines_str}",
            status = file.status,
            path = file.path,
        );
    }

    let needs_coverage: Vec<_> = files.iter().filter(|f| f.status != "OK").collect();
    if !needs_coverage.is_empty() {
        println!();
        println!("Files needing coverage:");
        for file in &needs_coverage {
            let misses_str =
                file.patch_misses
                    .map_or_else(|| "N/A".to_string(), |m| format!("{m}"));
            let base_str = format_coverage_pct(file.base_coverage);
            let head_str = format_coverage_pct(file.head_coverage);
            println!(
                "  {}: {misses_str} uncovered lines (was {base_str}, now {head_str})",
                file.path
            );
            if !file.uncovered_lines.is_empty() {
                let ranges = models::format_line_ranges(&file.uncovered_lines);
                println!("    lines: {ranges}");
            }
        }
    }
}

impl TextFormat for ChangedFileSummary {
    fn print_text(&self) {
        let patch_str = format_coverage_pct(self.patch_coverage);
        println!("[{}] {}: {patch_str}", self.status, self.path);
    }
}

impl TextFormat for FileReport {
    fn print_text(&self) {
        println!("File: {}", self.name.as_deref().unwrap_or("unknown"));
        if let Some(sha) = &self.commit_sha {
            println!("  Commit:     {sha}");
        }
        if let Some(totals) = &self.totals {
            if let Some(coverage) = totals.coverage {
                println!("  Coverage:   {coverage:.2}%");
            }
            if let Some(lines) = totals.lines {
                println!("  Lines:      {lines}");
            }
            if let Some(hits) = totals.hits {
                println!("  Hits:       {hits}");
            }
            if let Some(misses) = totals.misses {
                println!("  Misses:     {misses}");
            }
        }
    }
}

impl TextFormat for Flag {
    fn print_text(&self) {
        let name = self.flag_name.as_deref().unwrap_or("unknown");
        let coverage = self
            .coverage
            .map_or_else(|| "N/A".to_string(), |c| format!("{c:.2}%"));
        println!("{name}: {coverage}");
    }
}

impl TextFormat for Component {
    fn print_text(&self) {
        let name = self.name.as_deref().unwrap_or("unknown");
        let id = self.id.as_deref().unwrap_or("");
        let coverage = self
            .coverage
            .map_or_else(|| "N/A".to_string(), |c| format!("{c:.2}%"));
        println!("{name} ({id}): {coverage}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TotalsComparison;

    #[test]
    fn text_format_totals() {
        let totals = Totals {
            files: Some(10),
            lines: Some(100),
            hits: Some(80),
            misses: Some(15),
            partials: Some(5),
            coverage: Some(80.0),
            branches: Some(20),
            methods: Some(30),
            complexity: None,
            complexity_total: None,
            complexity_ratio: None,
            diff: None,
        };
        totals.print_text();
    }

    #[test]
    fn text_format_repo() {
        let repo = Repo {
            name: Some("my-repo".to_string()),
            private: Some(false),
            updatestamp: None,
            author: None,
            language: Some("rust".to_string()),
            branch: Some("main".to_string()),
            active: Some(true),
            activated: None,
            totals: Some(Totals {
                files: None,
                lines: None,
                hits: None,
                misses: None,
                partials: None,
                coverage: Some(85.5),
                branches: None,
                methods: None,
                complexity: None,
                complexity_total: None,
                complexity_ratio: None,
                diff: None,
            }),
        };
        repo.print_text();
    }

    #[test]
    fn text_format_flag() {
        let flag = Flag {
            flag_name: Some("unit".to_string()),
            coverage: Some(90.0),
        };
        flag.print_text();
    }

    #[test]
    fn text_format_comparison() {
        let cmp = Comparison {
            base_commit: Some("abc".to_string()),
            head_commit: Some("def".to_string()),
            totals: Some(TotalsComparison {
                base: Some(Totals {
                    files: None,
                    lines: None,
                    hits: None,
                    misses: None,
                    partials: None,
                    coverage: Some(80.0),
                    branches: None,
                    methods: None,
                    complexity: None,
                    complexity_total: None,
                    complexity_ratio: None,
                    diff: None,
                }),
                head: Some(Totals {
                    files: None,
                    lines: None,
                    hits: None,
                    misses: None,
                    partials: None,
                    coverage: Some(85.0),
                    branches: None,
                    methods: None,
                    complexity: None,
                    complexity_total: None,
                    complexity_ratio: None,
                    diff: None,
                }),
                patch: None,
            }),
            commit_uploads: None,
            diff: None,
            files: None,
            untracked: None,
        };
        cmp.print_text();
    }

    #[test]
    fn text_format_pr_summary() {
        let summary = PrSummary {
            pullid: 107,
            title: Some("Add feature".to_string()),
            state: Some("open".to_string()),
            ci_passed: Some(true),
            base_coverage: Some(89.50),
            head_coverage: Some(88.97),
            coverage_delta: Some(-0.53),
            patch_coverage: Some(39.79),
            patch_hits: Some(78),
            patch_lines: Some(196),
            status: "COVERAGE DECREASED".to_string(),
            changed_files: vec![
                ChangedFileSummary {
                    path: "src/client.rs".to_string(),
                    patch_coverage: Some(100.0),
                    patch_hits: Some(67),
                    patch_lines: Some(67),
                    patch_misses: Some(0),
                    base_coverage: Some(90.0),
                    head_coverage: Some(95.0),
                    status: "OK".to_string(),
                    uncovered_lines: vec![],
                },
                ChangedFileSummary {
                    path: "src/ops.rs".to_string(),
                    patch_coverage: Some(8.52),
                    patch_hits: Some(11),
                    patch_lines: Some(129),
                    patch_misses: Some(118),
                    base_coverage: Some(100.0),
                    head_coverage: Some(45.62),
                    status: "LOW COVERAGE".to_string(),
                    uncovered_lines: vec![10, 11, 12, 15, 20, 21, 22, 23, 24, 30],
                },
            ],
        };
        summary.print_text();
    }

    #[test]
    fn text_format_comparison_with_files() {
        use crate::models::{ComparisonFile, ComparisonFileName, FileTotals};

        let cmp = Comparison {
            base_commit: Some("abc".to_string()),
            head_commit: Some("def".to_string()),
            totals: None,
            commit_uploads: None,
            diff: None,
            files: Some(vec![
                ComparisonFile {
                    name: Some(ComparisonFileName {
                        base: Some("src/lib.rs".to_string()),
                        head: Some("src/lib.rs".to_string()),
                    }),
                    has_diff: Some(true),
                    stats: None,
                    totals: Some(FileTotals {
                        base: None,
                        head: None,
                        patch: Some(Totals {
                            files: None,
                            lines: Some(10),
                            hits: Some(8),
                            misses: Some(2),
                            partials: None,
                            coverage: Some(80.0),
                            branches: None,
                            methods: None,
                            complexity: None,
                            complexity_total: None,
                            complexity_ratio: None,
                            diff: None,
                        }),
                    }),
                    change_summary: None,
                    lines: None,
                },
                ComparisonFile {
                    name: Some(ComparisonFileName {
                        base: Some("src/utils.rs".to_string()),
                        head: Some("src/utils.rs".to_string()),
                    }),
                    has_diff: Some(false),
                    stats: None,
                    totals: None,
                    change_summary: None,
                    lines: None,
                },
            ]),
            untracked: None,
        };
        cmp.print_text();
    }

    #[test]
    fn json_output() {
        let flag = Flag {
            flag_name: Some("unit".to_string()),
            coverage: Some(90.0),
        };
        let json = serde_json::to_string_pretty(&flag).unwrap();
        assert!(json.contains("\"flag_name\""));
        assert!(json.contains("\"unit\""));
    }

    #[test]
    fn text_format_paginated() {
        let paginated = Paginated {
            count: Some(2),
            next: None,
            previous: None,
            results: vec![
                Flag {
                    flag_name: Some("unit".to_string()),
                    coverage: Some(90.0),
                },
                Flag {
                    flag_name: Some("integration".to_string()),
                    coverage: Some(75.0),
                },
            ],
            total_pages: Some(1),
        };
        paginated.print_text();
    }
}
