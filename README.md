# TubeSub

Desktop subtitle automator for YouTube creators. Built with Rust and GTK4/Adw-rs.

## Features

- OAuth2 authentication with YouTube
- Fetch and display your YouTube videos
- Upload SRT subtitle files to videos
- Drag-and-drop support for SRT files
- Dark theme support (Adwaita)

## Building

### Prerequisites

**Linux (Ubuntu/Debian):**
```bash
sudo apt install pkg-config libgtk-4-dev libadwaita-1-dev
```

**Windows (MSYS2):**
```bash
pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita mingw-w64-x86_64-pkg-config
```

### Build

```bash
cargo build --release
```

### Run

1. Create `.env` file:
```
YOUTUBE_CLIENT_ID=your-client-id
YOUTUBE_CLIENT_SECRET=your-client-secret
```

2. Run:
```bash
cargo run
```

## GitHub Actions

This project includes GitHub Actions workflows for automatic building:

- **build-windows.yml**: Builds on every push/PR to main
- **release.yml**: Creates a release when you push a version tag (v*)

### Creating a Release

1. Update version in `Cargo.toml`
2. Commit and push
3. Create and push a tag:
```bash
git tag v0.1.0
git push origin v0.1.0
```

4. GitHub Actions will automatically create a release with the Windows binary

## YouTube API Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project
3. Enable YouTube Data API v3
4. Create OAuth 2.0 credentials (Desktop application)
5. Set redirect URI to: `http://localhost:8080/callback`
6. Copy Client ID and Client Secret to `.env`

## License

MIT
