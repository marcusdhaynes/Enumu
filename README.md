# Enumu

Ultra-lightweight, offline-first batch file renamer for Windows, macOS, Linux, Android, iOS, and Web.

## Stack
Rust + Tauri 2 + Svelte + TypeScript. The shared UI includes browser previews; native filesystem renames use Rust commands in desktop builds.

## Safety
Preview operations before applying them. Preserve extensions unless explicitly requested. Native renaming checks collisions and avoids overwriting files. Browser mode is a preview-only foundation until permission-aware filesystem adapters are added.

## Development

```bash
npm install
npm run dev
```

For desktop builds, install Rust and the Tauri platform prerequisites, then run `npm run tauri dev`.

## Features
- Add/remove text from filename positions, beginning or end
- Sequential numbering, custom start and padding
- Hide extensions in previews without modifying them
- Collision checks and batch preview
- Cross-platform build automation in `.github/workflows`

## Planned
Permission-aware mobile filesystem access, scalable folder traversal, an operation journal for undo, fully tested collision-safe multi-file rename transactions.
