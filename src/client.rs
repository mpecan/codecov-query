use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header;
use serde::de::DeserializeOwned;

use crate::models::{
    Branch, Commit, Comparison, Component, FileReport, Flag, Paginated, Pull, Repo, TotalsResponse,
};

const BASE_URL: &str = "https://api.codecov.io/api/v2";

pub struct CodecovClient {
    client: Client,
    service: String,
    owner: String,
    repo: Option<String>,
}

impl CodecovClient {
    pub fn new(token: &str, service: &str, owner: &str, repo: Option<&str>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        let auth_value = header::HeaderValue::from_str(&format!("Bearer {token}"))
            .context("invalid token value")?;
        headers.insert(header::AUTHORIZATION, auth_value);
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self {
            client,
            service: service.to_string(),
            owner: owner.to_string(),
            repo: repo.map(ToString::to_string),
        })
    }

    fn base_url(&self) -> String {
        format!("{BASE_URL}/{}/{}", self.service, self.owner)
    }

    fn repo_url(&self) -> Result<String> {
        let repo = self
            .repo
            .as_deref()
            .context("--repo is required for this command")?;
        Ok(format!("{}/repos/{}", self.base_url(), repo))
    }

    fn handle_response<T: DeserializeOwned>(response: reqwest::blocking::Response) -> Result<T> {
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default(); // intentional: best-effort error body
            anyhow::bail!("API request failed (HTTP {status}): {body}");
        }
        let body = response.text().context("failed to read response body")?;
        serde_json::from_str(&body).context(format!("failed to deserialize response: {body}"))
    }

    pub fn list_repos(
        &self,
        active: Option<bool>,
        search: Option<&str>,
        names: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Paginated<Repo>> {
        let url = format!("{}/repos/", self.base_url());
        let mut request = self.client.get(&url);

        if let Some(active) = active {
            request = request.query(&[("active", active.to_string())]);
        }
        if let Some(search) = search {
            request = request.query(&[("search", search)]);
        }
        if let Some(names) = names {
            request = request.query(&[("names", names)]);
        }
        if let Some(page) = page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(page_size) = page_size {
            request = request.query(&[("page_size", page_size.to_string())]);
        }

        let response = request.send().context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn get_repo(&self) -> Result<Repo> {
        let url = format!("{}/", self.repo_url()?);
        let response = self
            .client
            .get(&url)
            .send()
            .context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn get_totals(
        &self,
        sha: Option<&str>,
        branch: Option<&str>,
        path: Option<&str>,
        flag: Option<&str>,
        component_id: Option<&str>,
    ) -> Result<TotalsResponse> {
        let url = format!("{}/totals/", self.repo_url()?);
        let mut request = self.client.get(&url);

        if let Some(sha) = sha {
            request = request.query(&[("sha", sha)]);
        }
        if let Some(branch) = branch {
            request = request.query(&[("branch", branch)]);
        }
        if let Some(path) = path {
            request = request.query(&[("path", path)]);
        }
        if let Some(flag) = flag {
            request = request.query(&[("flag", flag)]);
        }
        if let Some(component_id) = component_id {
            request = request.query(&[("component_id", component_id)]);
        }

        let response = request.send().context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn list_commits(
        &self,
        branch: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Paginated<Commit>> {
        let url = format!("{}/commits/", self.repo_url()?);
        let mut request = self.client.get(&url);

        if let Some(branch) = branch {
            request = request.query(&[("branch", branch)]);
        }
        if let Some(page) = page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(page_size) = page_size {
            request = request.query(&[("page_size", page_size.to_string())]);
        }

        let response = request.send().context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn get_commit(&self, commitid: &str) -> Result<Commit> {
        let url = format!("{}/commits/{}/", self.repo_url()?, commitid);
        let response = self
            .client
            .get(&url)
            .send()
            .context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn list_branches(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Paginated<Branch>> {
        let url = format!("{}/branches/", self.repo_url()?);
        let mut request = self.client.get(&url);

        if let Some(page) = page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(page_size) = page_size {
            request = request.query(&[("page_size", page_size.to_string())]);
        }

        let response = request.send().context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn get_branch(&self, name: &str) -> Result<Branch> {
        let url = format!("{}/branches/{}/", self.repo_url()?, name);
        let response = self
            .client
            .get(&url)
            .send()
            .context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn list_pulls(
        &self,
        state: Option<&str>,
        start_date: Option<&str>,
        ordering: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Paginated<Pull>> {
        let url = format!("{}/pulls/", self.repo_url()?);
        let mut request = self.client.get(&url);

        if let Some(state) = state {
            request = request.query(&[("state", state)]);
        }
        if let Some(start_date) = start_date {
            request = request.query(&[("start_date", start_date)]);
        }
        if let Some(ordering) = ordering {
            request = request.query(&[("ordering", ordering)]);
        }
        if let Some(page) = page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(page_size) = page_size {
            request = request.query(&[("page_size", page_size.to_string())]);
        }

        let response = request.send().context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn get_pull(&self, pullid: u64) -> Result<Pull> {
        let url = format!("{}/pulls/{}/", self.repo_url()?, pullid);
        let response = self
            .client
            .get(&url)
            .send()
            .context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn compare(
        &self,
        base: Option<&str>,
        head: Option<&str>,
        pullid: Option<u64>,
    ) -> Result<Comparison> {
        let url = format!("{}/compare/", self.repo_url()?);
        let mut request = self.client.get(&url);

        if let Some(base) = base {
            request = request.query(&[("base", base)]);
        }
        if let Some(head) = head {
            request = request.query(&[("head", head)]);
        }
        if let Some(pullid) = pullid {
            request = request.query(&[("pullid", pullid.to_string())]);
        }

        let response = request.send().context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn get_file_report(
        &self,
        path: &str,
        sha: Option<&str>,
        branch: Option<&str>,
    ) -> Result<FileReport> {
        let url = format!("{}/file_report/{}/", self.repo_url()?, path);
        let mut request = self.client.get(&url);

        if let Some(sha) = sha {
            request = request.query(&[("sha", sha)]);
        }
        if let Some(branch) = branch {
            request = request.query(&[("branch", branch)]);
        }

        let response = request.send().context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn list_flags(&self, page: Option<u32>, page_size: Option<u32>) -> Result<Paginated<Flag>> {
        let url = format!("{}/flags/", self.repo_url()?);
        let mut request = self.client.get(&url);

        if let Some(page) = page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(page_size) = page_size {
            request = request.query(&[("page_size", page_size.to_string())]);
        }

        let response = request.send().context("failed to send request")?;
        Self::handle_response(response)
    }

    pub fn list_components(&self) -> Result<Paginated<Component>> {
        let url = format!("{}/components/", self.repo_url()?);
        let response = self
            .client
            .get(&url)
            .send()
            .context("failed to send request")?;
        Self::handle_response(response)
    }
}
