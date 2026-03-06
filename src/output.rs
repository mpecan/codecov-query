use crate::models::{
    Branch, Commit, Comparison, Component, FileReport, Flag, Paginated, Pull, Repo, Totals,
    TotalsResponse,
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
