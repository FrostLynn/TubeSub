# Product Requirement Document (PRD)

## Project Title: TubeSub (Desktop Subtitle Automator)
**Status:** Draft  
**Target Platform:** Windows 10 / 11 (x86_64, aarch64)  
**Tech Stack:** Rust (Backend & Core), Adw-rs / GTK4 (Frontend UI via Windows Native rendering pipeline)

---

## 1. Executive Summary & Goals
TubeSub is a lightweight desktop utility designed to eliminate the manual friction of managing closed captions across YouTube channels. Instead of navigating through multiple nested menus inside YouTube Studio for every single upload, creators can authenticate locally, view a clear listing of their recent videos, and map local `.srt` files directly to targeted video assets.

### Core Objectives
* **Efficiency:** Reduce the operational time needed to match and upload subtitle tracks to multiple videos by at least 80%.
* **Developer/Power-User UX:** Provide a clean, distraction-free environment utilizing the GNOME design philosophy (flat structural hierarchies, unified header bars, elegant typography) compiled natively as a standalone application.
* **Safety:** Zero cloud or third-party storage of sensitive user tokens; all sessions and OAuth authorization routines are handled entirely on the client machine via Rust's memory-safe runtime.

---

## 2. User Persona & Design Philosophy

### User Persona
* **The High-Output Creator:** Uploads episodic videos, technical documentation guides, or multi-language content weekly. Requires an interface that acts as a predictable, lightning-fast utility rather than a complex content dashboard ecosystem.

### GNOME Design Guidelines on Windows
To cleanly bridge the gap between GNOME’s human interface design language and the native Windows operating system:
* **The HeaderBar (Adwaita Pattern):** Combines the traditional title bar and application window toolbar into a single, cohesive header. Window control buttons (Close, Minimize, Maximize) respect standard Windows positions, but primary operational actions (Fetch, Profile management) reside directly inside the HeaderBar area.
* **Content View & Scannable Lists:** Use well-padded, rounded action rows (`AdwActionRow` layout paradigm) for displaying video assets. Avoid heavy, unstyled spreadsheet tables; prioritize immediate readability with explicit trailing status indicators.
* **Dark Theme First:** Support an explicit, system-compliant dark theme option using the refined Adwaita dark variant palette, mapped smoothly onto the Windows window canvas.

---

## 3. Functional Requirements

### FR-1: Local OAuth2 Session Management
* **Description:** The system must securely initialize a local loopback listener server to receive the Google OAuth verification handshake.
* **Rust Implementation:** Leverage the `yup-oauth2` or `google-youtube3` ecosystem to safely isolate, encrypt, and handle token refresh parameters.
* **UI Experience:** A clean GNOME-style *Status Page* showing a disconnected state with a single primary action card: "Sign In via Default Browser".

### FR-2: Streamed Uploads Playlist Resolution
* **Description:** The program must resolve the user's channel metadata to locate its hidden, system-managed "Uploads" playlist reference.
* **API Chain Pipeline:**
  1. `channels.list(mine=true)` &rarr; extracts `contentDetails.relatedPlaylists.uploads`
  2. `playlistItems.list(playlistId=...)` &rarr; parses individual video titles, assets, and unique Video IDs.
* **UI Experience:** A flat, vertical list display utilizing lazy loading indicators natively embedded into the scroll space.

### FR-3: Drag-and-Drop Single/Bulk Mapping
* **Description:** Users can click a list row item to bring up a native file attachment sheet or drop an `.srt` file directly over the target video row.
* **API Transaction:** `captions.insert(part="snippet")` passing a raw binary payload stream formatted as `application/octet-stream`.

---

## 4. Technical Architecture & Constraints

```
+-------------------------------------------------------------+
|                        GTK4 / Adw-rs                        |
|             (GNOME Design UI Layout Component)             |
+----------------------------------------------+--------------+
                                               | (Events/Channels)
                                               v
+-------------------------------------------------------------+
|                      Tokio Async Core                       |
|          (Handles Network I/O and File System Access)       |
+----------------------+-----------------------+--------------+
                       |                       |
                       v                       v
         +---------------------------+   +---------------------------+
         |      reqwest / hyper      |   |        yup-oauth2         |
         |  (YouTube Data API v3)    |   |  (Local Secret Storage)   |
         +---------------------------+   +---------------------------+
```

### Stack Selection Rationale
* **GUI Engine (`gtk4` / `adw-rs`):** Delivers the exact visual fidelity of the Adwaita design catalog directly within Rust. It compiles cleanly for Windows environments using the MSYS2 toolchain ecosystem, bundling required theme assets inside the target binary.
* **Async Runtime (`tokio`):** Guarantees that multi-megabyte caption uploads or network pagination routines never interrupt or block the main GTK interface rendering loop.

---

## 5. UI Layout Wireframe Blueprint

The main window establishes a unified, single-pane display schema that avoids nested modal layers:

```
+-----------------------------------------------------------+
| [Profile]               TubeSub             [-] [[]] [X] |  <- GNOME Unified HeaderBar
+-----------------------------------------------------------+
|                                                           |
|  +-----------------------------------------------------+  |
|  |  (i) Authenticated as: CreativeChannel101           |  |  <- AdwActionRow banner
|  +-----------------------------------------------------+  |
|                                                           |
|  Recent Uploaded Videos                       [ Refresh ] |
|  +-----------------------------------------------------+  |
|  | [Img] Building an Emulator in C             (Add SRT) |  |  <- Row 1
|  | [Img] Self-hosting with Docker on Armbian   (Add SRT) |  |  <- Row 2
|  | [Img] React vs Vite Performance Review      (Add SRT) |  |  <- Row 3
|  |                                                     |  |
|  |                                                     |  |
|  +-----------------------------------------------------+  |
|                                                           |
+-----------------------------------------------------------+
```

---

## 6. Non-Functional Requirements (NFR)

* **Performance:** The target binary footprint must be highly efficient, staying under **35MB** using release compilation optimizations (`opt-level = 3`, `lto = true`).
* **Memory Architecture:** Zero structural memory leaks across repeated network polling tasks or interface refresh loops.
* **Platform Isolation:** While configured for native compilation under Windows targets (`x86_64-pc-windows-msvc`), backend business routines must be strictly uncoupled from frontend window logic to keep cross-platform compilation pristine.
* **API Quota Mitigation:** The application will temporarily cache session video queries in-memory to prevent accidental drain of the developer account's standard daily 10,000 unit YouTube API budget.