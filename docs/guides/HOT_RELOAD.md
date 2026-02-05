# AgentKern Hot-Reload Guide

For rapid development, you can enable hot-reloading for both the Rust server and the TypeScript playground.

## 1. Rust Server (using `cargo-watch`)

`cargo-watch` monitors your source files and automatically restarts the server when it detects changes.

### Installation
```bash
cargo install cargo-watch
```

### Usage
```bash
# Watches all files in the workspace and runs the server
cargo watch -x "run -p agentkern-server"
```

---

## 2. TypeScript Playground (using `Vite`)

The playground is built on Vite, which supports Hot Module Replacement (HMR) out of the box.

### Usage
```bash
cd apps/playground
npm run dev
```
Changes to React components will be reflected instantly in the browser without a full page reload.

---

## 3. Native Bindings (Re-building)

If you modify the Rust code in `packages/foundation/native-binding`, you must re-build the N-API module for the playground to see the changes.

```bash
# In the root or playground directory
pnpm install # Triggers a re-build via turbo
```
Or manually:
```bash
cd packages/foundation/native-binding
pnpm build
```
Then restart the playground if it doesn't pick up the new `.node` file.
