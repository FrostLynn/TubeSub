use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChannelResponse {
    items: Vec<ChannelItem>,
}

#[derive(Debug, Deserialize)]
struct ChannelItem {
    #[serde(rename = "contentDetails")]
    content_details: ContentDetails,
}

#[derive(Debug, Deserialize)]
struct ContentDetails {
    #[serde(rename = "relatedPlaylists")]
    related_playlists: RelatedPlaylists,
}

#[derive(Debug, Deserialize)]
struct RelatedPlaylists {
    uploads: String,
}

#[derive(Debug, Deserialize)]
struct PlaylistItemsResponse {
    items: Vec<PlaylistItem>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlaylistItem {
    snippet: Snippet,
}

#[derive(Debug, Deserialize)]
struct Snippet {
    title: String,
    #[serde(rename = "resourceId")]
    resource_id: ResourceId,
    thumbnails: Option<Thumbnails>,
}

#[derive(Debug, Deserialize)]
struct ResourceId {
    #[serde(rename = "videoId")]
    video_id: String,
}

#[derive(Debug, Deserialize)]
struct Thumbnails {
    default: Option<Thumbnail>,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: String,
}

#[derive(Debug)]
pub struct YouTubeClient {
    access_token: String,
    http: reqwest::blocking::Client,
    cached_videos: std::sync::RwLock<Vec<Video>>,
}

impl YouTubeClient {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            http: reqwest::blocking::Client::new(),
            cached_videos: std::sync::RwLock::new(Vec::new()),
        }
    }

    pub fn fetch_videos(&self) -> Result<Vec<Video>> {
        {
            let cache = self.cached_videos.read().unwrap();
            if !cache.is_empty() {
                return Ok(cache.clone());
            }
        }

        let uploads_playlist_id = self.get_uploads_playlist_id()?;
        let videos = self.get_playlist_items(&uploads_playlist_id)?;
        *self.cached_videos.write().unwrap() = videos.clone();
        Ok(videos)
    }

    fn get_uploads_playlist_id(&self) -> Result<String> {
        let resp = self
            .http
            .get("https://www.googleapis.com/youtube/v3/channels")
            .bearer_auth(&self.access_token)
            .query(&[("part", "contentDetails"), ("mine", "true")])
            .send()?;

        let status = resp.status();
        let text = resp.text()?;

        if !status.is_success() {
            anyhow::bail!("YouTube API error ({}): {}", status, text);
        }

        let channel_resp: ChannelResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse channel response: {}. Body: {}", e, text))?;

        Ok(channel_resp
            .items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No channel found"))?
            .content_details
            .related_playlists
            .uploads)
    }

    fn get_playlist_items(&self, playlist_id: &str) -> Result<Vec<Video>> {
        let mut videos = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut params = vec![
                ("part", "snippet"),
                ("playlistId", playlist_id),
                ("maxResults", "50"),
            ];

            if let Some(ref token) = page_token {
                params.push(("pageToken", token));
            }

            let resp = self
                .http
                .get("https://www.googleapis.com/youtube/v3/playlistItems")
                .bearer_auth(&self.access_token)
                .query(&params)
                .send()?;

            let status = resp.status();
            let text = resp.text()?;

            if !status.is_success() {
                anyhow::bail!("YouTube API error ({}): {}", status, text);
            }

            let playlist_resp: PlaylistItemsResponse = serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("Failed to parse playlist response: {}. Body: {}", e, text))?;

            for item in playlist_resp.items {
                let thumbnail_url = item
                    .snippet
                    .thumbnails
                    .and_then(|t| t.default)
                    .map(|d| d.url);

                videos.push(Video {
                    id: item.snippet.resource_id.video_id,
                    title: item.snippet.title,
                    thumbnail_url,
                });
            }

            page_token = playlist_resp.next_page_token;
            if page_token.is_none() {
                break;
            }
        }

        Ok(videos)
    }

    pub fn upload_caption(&self, video_id: &str, _srt_content: &str, language: &str) -> Result<()> {
        let url = "https://www.googleapis.com/youtube/v3/captions?part=snippet";

        let body = serde_json::json!({
            "snippet": {
                "videoId": video_id,
                "language": language,
                "name": format!("{} - {}", language, video_id),
                "isDraft": false
            }
        });

        let resp = self
            .http
            .post(url)
            .bearer_auth(&self.access_token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text()?;
            anyhow::bail!("Upload caption error ({}): {}", status, text);
        }

        Ok(())
    }

    pub fn clear_cache(&self) {
        self.cached_videos.write().unwrap().clear();
    }
}
