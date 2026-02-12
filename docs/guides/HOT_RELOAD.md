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

## 3. SDK Rebuild (Optional)

If you modify SDK Rust crates, rebuild the SDK package before local verification.

```bash
pnpm --dir sdks/node build
```

Then re-run your SDK smoke checks.
