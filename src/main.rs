use std::process;
use regex::Regex;

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;

const DNS_RECORDS_URL: &'static str = "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records";

#[derive(Deserialize)]
struct Config {
    freshtomato: FreshTomato,
    cloudflare: Cloudflare,
}

#[derive(Deserialize)]
struct FreshTomato {
    username: String,
    password: String,
    url: String,
}

#[derive(Deserialize)]
struct Cloudflare {
    api_key: String,
    zone_id: String,
}

struct ARecord {
    id: String,
    name: String,
    content: String,
}

fn main() {
    let config = get_config();

    let wan_ip = get_wan_ip(&config.freshtomato);
    println!("Current WAN ID: {}", &wan_ip);

    let a_records = get_a_records(&config.cloudflare);
    for a_record in a_records {
        if a_record.content != wan_ip {
            println!("Updating record: {}", &a_record.name);
            update_a_record(&config.cloudflare, &a_record.id, &wan_ip)
        } else {
            println!("A Record for {} is up to date", &a_record.name);
        }
    }
}

fn get_config() -> Config {
    let config_str = std::fs::read_to_string("config.toml")
        .expect("Unable to read config file. Does it exist? Is it readable?");
    let config: Config = toml::from_str(&config_str)
        .expect("Unable to parse config file. Is it valid toml? Does it have all fields?");
    config
}

fn get_wan_ip(tomato_conf: &FreshTomato) -> String {
    // Fetch the FreshTomato router homepage
    let client = Client::new();
    let response = client.get(&tomato_conf.url)
        .basic_auth(&tomato_conf.username, Some(&tomato_conf.password))
        .send().unwrap();

    if !response.status().is_success() {
        println!("Failed to load the FreshTomato router homepage");
        process::exit(1);
    }

    let html = response.text().unwrap();
    
    // Find the value of http_id in the html variable
    let re = Regex::new(r#"http_id=([^"]+)"#).unwrap();
    let http_id:&str;
    if let Some(captures) = re.captures(&html) {
        http_id = captures.get(1).unwrap().as_str();
    } else {
        println!("Unable to find http_id");
        process::exit(1);
    }

    let info_url = format!("{}/status-data.jsx?_http_id={}", &tomato_conf.url, http_id);

    let response = client.get(info_url)
        .basic_auth(&tomato_conf.username, Some(&tomato_conf.password))
        .send().unwrap();

    if !response.status().is_success() {
        println!("Failed to load the FreshTomato status page");
        process::exit(1);
    }

    let html = response.text().unwrap();

    let re = Regex::new(r#"'wan_ipaddr':\s*'([^']+)'"#).unwrap();

    let wan_ip:&str;
    if let Some(captures) = re.captures(&html) {
        wan_ip = captures.get(1).unwrap().as_str();
    } else {
        println!("Unable to find WAN IP address");
        process::exit(1);
    }

    wan_ip.to_string()
}

fn get_a_records(cloudflare: &Cloudflare) -> Vec<ARecord> {
    let url = DNS_RECORDS_URL.replace("{zone_id}", &cloudflare.zone_id);

    let client = Client::new();

    let bearer = create_bearer(&cloudflare.api_key);
    
    let response = client.get(&url)
        .header("Authorization", &bearer)
        .send().unwrap();

    let status = response.status();
    if !status.is_success() {
        let status = status.as_str();
        println!("Failed to load the existing Cloudflare DNS records. Status: {}", status);
        process::exit(1);
    }

    let json: serde_json::Value = response.json().unwrap();
    let records = json["result"].as_array().unwrap();

    let mut record_ids = Vec::new();
    for record in records {
        if record["type"] == "A" {
            let id = record["id"].as_str().unwrap().to_string();
            let name = record["name"].as_str().unwrap().to_string();
            let content = record["content"].as_str().unwrap().to_string();
            let a_record = ARecord { id, name, content };

            record_ids.push(a_record);
        }
    }

    record_ids
}

fn update_a_record(cloudflare: &Cloudflare, record_id: &String, ip: &String) {
    println!("Updating A record ID: {}", record_id);
    let mut url = DNS_RECORDS_URL.replace("{zone_id}", &cloudflare.zone_id);
    url.push_str(&record_id);

    println!("URL: {}", url);

    let client = Client::new();

    let bearer = create_bearer(&cloudflare.api_key);
    let body = json!({
            "content": ip,
        }).to_string();

    let response = client.patch(&url)
        .header("Authorization", &bearer)
        .body(body)
        .send().unwrap();

    let status = response.status();
    if !status.is_success() {
        let status = status.as_str();
        println!("Failed to load the existing Cloudflare DNS records. Status: {}", status);
        println!("Response: {:?}", response.text());
        process::exit(1);
    }
}

fn create_bearer(api_key: &str) -> String {
    format!("Bearer {}", api_key)
}