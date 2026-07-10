use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Debug)]
pub struct OAuthManager {
    config_dir: PathBuf,
    client_id: String,
    client_secret: String,
}

impl OAuthManager {
    pub fn new(client_id: String, client_secret: String) -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("tubesub");

        std::fs::create_dir_all(&config_dir)?;
        Ok(Self {
            config_dir,
            client_id,
            client_secret,
        })
    }

    pub fn token_path(&self) -> PathBuf {
        self.config_dir.join("token.json")
    }

    pub fn load_token(&self) -> Option<TokenData> {
        let path = self.token_path();
        if !path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save_token(&self, token: &TokenData) -> Result<()> {
        let path = self.token_path();
        let content = serde_json::to_string_pretty(token)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_auth_url(&self) -> String {
        let redirect_uri = "http://localhost:8080/callback";

        format!(
            "https://accounts.google.com/o/oauth2/auth?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/youtube.force-ssl&access_type=offline&prompt=consent",
            self.client_id, redirect_uri
        )
    }

    pub fn exchange_code(&self, code: &str) -> Result<TokenData> {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("code", code),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("redirect_uri", "http://localhost:8080/callback"),
                ("grant_type", "authorization_code"),
            ])
            .send()?
            .json::<serde_json::Value>()?;

        let access_token = resp["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No access token in response"))?
            .to_string();

        let refresh_token = resp["refresh_token"].as_str().map(|s| s.to_string());
        let expires_in = resp["expires_in"].as_u64();

        let token = TokenData {
            access_token,
            refresh_token,
            expires_at: expires_in.map(|e| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + e
            }),
        };

        self.save_token(&token)?;
        Ok(token)
    }

    pub fn start_auth_flow(&self) -> Result<TokenData> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        listener.set_nonblocking(true)?;

        let auth_url = self.get_auth_url();
        open::that(&auth_url)?;

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut reader = BufReader::new(&stream);
                    let mut request_line = String::new();
                    reader.read_line(&mut request_line)?;

                    if let Some(code) = extract_code(&request_line) {
                        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authentication successful!</h1><p>You can close this window.</p></body></html>";
                        stream.write_all(response.as_bytes())?;

                        return self.exchange_code(&code);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

fn extract_code(request: &str) -> Option<String> {
    let parts: Vec<&str> = request.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let url = parts[1];
    let query_start = url.find('?')?;
    let query = &url[query_start + 1..];

    for param in query.split('&') {
        let mut kv = param.splitn(2, '=');
        if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
            if key == "code" {
                return Some(value.to_string());
            }
        }
    }

    None
}
