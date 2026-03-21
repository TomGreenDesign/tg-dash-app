# TG Dash

Native desktop wrapper for [dash.tomgreen.uk](https://dash.tomgreen.uk) built with [Tauri v2](https://v2.tauri.app).

## Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js 22+](https://nodejs.org/)
- Platform-specific Tauri dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```

Produces platform-native installers in `src-tauri/target/release/bundle/`:
- **macOS**: `.dmg`
- **Windows**: `.msi` / `.exe`
- **Linux**: `.deb` / `.AppImage`

## CI

GitHub Actions builds for all three platforms on push to `main`. Download artifacts from the Actions tab.
