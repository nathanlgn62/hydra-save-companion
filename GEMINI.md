# Hydra Save Companion - Guidelines & Architecture

## Project Overview

Hydra Save Companion is a desktop utility built with Tauri (Rust + React/TypeScript) designed to automatically track, backup, and synchronize PC video game saves locally and across multiple cloud providers.

## Tech Stack

- **Desktop Framework:** Tauri (v2)
- **Backend:** Rust (tokio, reqwest, serde, zip, walkdir, chrono)
- **Frontend:** React, TypeScript, Tailwind CSS
- **Data Sources:** Ludasavi game save manifest integration for automatic path resolution.

## Architecture & Core Features

### 1. Multi-Cloud & Unified Authentication

- **Unified Flow:** OAuth authentication is handled via a shared Rust command (`login_cloud`) running a local loopback server to intercept authorization codes.
- **Providers:**
  - **Google Drive:** Fully implemented (supports OAuth token refresh, folder auto-creation, upload/download, telemetry, and 5-save rotation).
  - **Dropbox:** Fully implemented (supports OAuth2 with refresh tokens, folder auto-creation, upload/download with exact client_modified save timestamp preservation, and storage quota telemetry).
  - **Proton Drive:** Disabled in UI due to absence of public third-party OAuth API (local synchronized folder recommended).

### 2. Backup & Rotation Strategy (5-Save History)

- **Google Drive Rotation:** Instead of overwriting files, the backend maintains unique file instances and applies a rotation policy.
- **Retention Limit:** Automatically queries remote files sorted by creation time (`createdTime desc`) and purges files exceeding the 5 most recent backups per game.

### 3. Save Path Resolution (Ludasavi)

- Resolves dynamic placeholders (e.g., `%AppData%`, Steam IDs) using a cached Ludasavi manifest to locate local game saves automatically or fall back to custom user paths.

## Development & Test Commands

- **Install dependencies:** `pnpm install`
- **Run frontend tests:** `pnpm test`
- **Run Rust tests:** `cargo test --manifest-path src-tauri/Cargo.toml`
- **Run in development mode (Tauri):** `pnpm run tauri dev`
- **Build for production:** `pnpm tauri build`

## AI Development Guidelines

1. **File Naming Convention (Strict):**
   - **All project files without exception must strictly follow the kebab-case naming convention (`kebab-case.extension`).**
   - PascalCase, camelCase, or snake_case file names are strictly forbidden (e.g., `game-card.tsx`, `use-sync-status.ts`, `auth-provider.ts`, `cloud-service.rs`, `sync-dialog.tsx`).
2. **Rust Backend (`src-tauri/`):**
   - Ensure all Tauri commands return `Result<T, String>` for robust error handling propagated to the frontend.
   - Handle asynchronous operations using `tokio` and non-blocking HTTP requests with `reqwest`.
   - Use `#[serde(rename_all = "camelCase")]` on data transfer structs to match TypeScript conventions.
3. **Frontend & State:**
   - Keep strict type safety between Rust models and TypeScript interfaces.
   - Maintain reactive UI state for sync status indicators (`UpToDate`, `LocalNewer`, `CloudNewer`, `NotFound`).
4. **Code Identifier Naming Conventions:**
   - Rust: `snake_case` for functions, variables, and modules; `PascalCase` for structs and enums.
   - TypeScript/React: `camelCase` for variables, functions, and hooks; `PascalCase` for components and types/interfaces (even though the file containing the component is named in `kebab-case.tsx`).

### Automatic Activation Rule: frontend-design

Whenever a task involves creating, redesigning, or modifying user interface (UI) elements, visual components, CSS/Tailwind styles, or front-end pages:

1. Automatically activate the directives defined in `.agents/skills/frontend-design/SKILL.md` (located at the root of the repository).
2. Do not generate any UI code without first validating the aesthetic intent and the DFII assessment.
3. You must strictly adhere to the `kebab-case.extension` file naming convention for any new component or generated file.
4. For purely logical tasks (algorithms, business logic, APIs, CI/CD), ignore this skill and answer directly.
