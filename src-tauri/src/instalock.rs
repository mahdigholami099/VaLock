use base64::Engine;
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::errors;

const PLATFORM: &str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";

pub struct InstaLock {
    client: Client,
    api_base: String,
    headers: reqwest::header::HeaderMap,
    agents: HashMap<String, String>,
    maps: HashMap<String, String>,
    config: HashMap<String, String>,
}

impl InstaLock {
    pub fn new(config: HashMap<String, String>) -> Result<Self, errors::MyErr> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let agents = Self::fetch_agents(&client)?;
        let maps = Self::fetch_maps(&client)?;
        let headers = Self::build_headers(&client)?;

        Ok(Self {
            client,
            api_base: "https://glz-eu-1.eu.a.pvp.net".to_string(),
            headers,
            agents,
            maps,
            config,
        })
    }

    fn fetch_agents(client: &Client) -> Result<HashMap<String, String>, errors::MyErr> {
        let resp: Value = client
            .get("https://valorant-api.com/v1/agents")
            .send()?
            .json()?;
        let mut agents = HashMap::new();
        if let Some(data) = resp["data"].as_array() {
            for agent in data {
                if let (Some(name), Some(uuid)) =
                    (agent["displayName"].as_str(), agent["uuid"].as_str())
                {
                    agents.insert(name.to_string(), uuid.to_string());
                }
            }
        }
        Ok(agents)
    }

    fn fetch_maps(client: &Client) -> Result<HashMap<String, String>, errors::MyErr> {
        let resp: Value = client
            .get("https://valorant-api.com/v1/maps")
            .send()?
            .json()?;
        let mut maps = HashMap::new();
        if let Some(data) = resp["data"].as_array() {
            for map in data {
                if let (Some(url), Some(name)) =
                    (map["mapUrl"].as_str(), map["displayName"].as_str())
                {
                    maps.insert(url.to_string(), name.to_string());
                }
            }
        }
        Ok(maps)
    }

    fn parse_lockfile() -> Result<Vec<String>, errors::MyErr> {
        let local_appdata =
            std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set");
        let lockfile_path = PathBuf::from(local_appdata)
            .join("Riot Games")
            .join("Riot Client")
            .join("Config")
            .join("lockfile");

        let content = fs::read_to_string(&lockfile_path)?;
        let parts: Vec<String> = content.split(':').map(String::from).collect();
        Ok(parts)
    }

    fn get_bearer(lockfile: &[String]) -> String {
        let password = format!("riot:{}", lockfile[3]);
        base64::engine::general_purpose::STANDARD.encode(password.as_bytes())
    }

    fn get_entitlements(client: &Client, bearer: &str, port: &str) -> Result<Value, errors::MyErr> {
        let url = format!("https://127.0.0.1:{}/entitlements/v1/token", port);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Basic {}", bearer).parse().unwrap(),
        );
        headers.insert("X-Riot-ClientPlatform", PLATFORM.parse().unwrap());
        headers.insert(
            "User-Agent",
            "PostmanRuntime/7.28.0".parse().unwrap(),
        );
        headers.insert("Accept-Encoding", "br".parse().unwrap());

        let resp: Value = client
            .get(&url)
            .headers(headers)
            .send()?
            .json()?;
        Ok(resp)
    }

    fn fetch_version(client: &Client) -> Result<String, errors::MyErr> {
        let resp: Value = client
            .get("https://valorant-api.com/v1/version")
            .send()?
            .json()?;
        let data = &resp["data"];
        let branch = data["branch"].as_str().unwrap_or("release");
        let build = data["buildVersion"].as_str().unwrap_or("0");
        let version_parts = data["version"]
            .as_str()
            .unwrap_or("0.0.0.0")
            .split('.')
            .collect::<Vec<&str>>();
        let patch = version_parts.last().unwrap_or(&"0");
        Ok(format!("{}-shipping-{}-{}", branch, build, patch))
    }

    fn build_headers(client: &Client) -> Result<reqwest::header::HeaderMap, errors::MyErr> {
        let lockfile = Self::parse_lockfile()?;
        let bearer = Self::get_bearer(&lockfile);
        let port = &lockfile[2];

        let entitlements = Self::get_entitlements(client, &bearer, port)?;
        let access_token = entitlements["accessToken"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let token = entitlements["token"].as_str().unwrap_or("").to_string();
        let version = Self::fetch_version(client)?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        headers.insert("X-Riot-Entitlements-JWT", token.parse().unwrap());
        headers.insert("X-Riot-ClientVersion", version.parse().unwrap());
        headers.insert("X-Riot-ClientPlatform", PLATFORM.parse().unwrap());

        Ok(headers)
    }

    fn read_log_urls(log_path: &PathBuf) -> Vec<String> {
        let mut urls = Vec::new();
        if let Ok(file) = fs::File::open(log_path) {
            let reader = BufReader::new(file);
            let re = Regex::new(r"https?://[^\s]+").unwrap();
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some(mat) = re.find(&line) {
                        let mut url = mat.as_str().to_string();
                        if url.ends_with("],") {
                            url = url[..url.len() - 2].to_string();
                        }
                        urls.push(url);
                    }
                }
            }
        }
        urls
    }

    fn get_log_path() -> PathBuf {
        let local_appdata =
            std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set");
        PathBuf::from(local_appdata)
            .join("VALORANT")
            .join("Saved")
            .join("Logs")
            .join("ShooterGame.log")
    }

    pub fn run<F: Fn(String) + Send + 'static>(
        &self,
        status_callback: F,
    ) -> Result<(), errors::MyErr> {
        let log_path = Self::get_log_path();
        let pregame_prefix = format!("{}/pregame/v1/matches/", self.api_base);
        let core_prefix = format!("{}/core-game/v1/matches/", self.api_base);

        let mut printed = false;

        loop {
            let urls = Self::read_log_urls(&log_path);

            let mut pregame_matches: Vec<String> = Vec::new();
            let mut core_matches: Vec<String> = Vec::new();

            for url in &urls {
                if url.contains(&pregame_prefix)
                    && !url.contains("/chattoken")
                    && !url.contains("/voicetoken")
                    && !url.contains("/loadouts")
                    && !url.contains("/select")
                    && !url.contains("/lock")
                {
                    let id = url.replace(&pregame_prefix, "");
                    pregame_matches.push(id);
                }
                if url.contains(&core_prefix)
                    && !url.contains("/teamchatmuctoken")
                    && !url.contains("/allchatmuctoken")
                    && !url.contains("/loadouts")
                    && !url.contains("/lock")
                    && !url.contains("/select")
                {
                    let id = url.replace(&core_prefix, "");
                    core_matches.push(id);
                }
            }

            if !pregame_matches.is_empty() {
                let match_id = pregame_matches.last().unwrap().clone();

                if !core_matches.contains(&match_id) {
                    status_callback("Match detected, locking agent...".to_string());

                    let match_url = format!(
                        "{}/pregame/v1/matches/{}",
                        self.api_base, match_id
                    );
                    let match_data: Value = self
                        .client
                        .get(&match_url)
                        .headers(self.headers.clone())
                        .send()?
                        .json()?;

                    let map_id = match_data["MapID"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();

                    let map_name = self
                        .maps
                        .get(&map_id)
                        .cloned()
                        .unwrap_or_default();

                    let agent_name = self
                        .config
                        .get(&map_name)
                        .cloned()
                        .unwrap_or_default();

                    let agent_uuid = self
                        .agents
                        .get(&agent_name)
                        .cloned()
                        .unwrap_or_default();

                    if agent_uuid.is_empty() {
                        status_callback(format!(
                            "No agent configured for map: {}",
                            map_name
                        ));
                        return Ok(());
                    }

                    thread::sleep(Duration::from_secs(1));

                    let select_url = format!(
                        "{}/pregame/v1/matches/{}/select/{}",
                        self.api_base, match_id, agent_uuid
                    );
                    let _: Value = self
                        .client
                        .post(&select_url)
                        .headers(self.headers.clone())
                        .send()?
                        .json()?;

                    status_callback(format!(
                        "Selected {} for map {}",
                        agent_name, map_name
                    ));

                    thread::sleep(Duration::from_secs(1));

                    let lock_url = format!(
                        "{}/pregame/v1/matches/{}/lock/{}",
                        self.api_base, match_id, agent_uuid
                    );
                    let _: Value = self
                        .client
                        .post(&lock_url)
                        .headers(self.headers.clone())
                        .send()?
                        .json()?;

                    status_callback(format!(
                        "Locked {}! Good luck!",
                        agent_name
                    ));

                    return Ok(());
                }
            } else if !printed {
                printed = true;
                status_callback("Waiting for match...".to_string());
            }

            thread::sleep(Duration::from_secs(1));
        }
    }
}
