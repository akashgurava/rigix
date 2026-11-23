# 0002: Solid 2.0 Web Client, Native Primitives & Tauri Embedded Shell

## Context
Rigix requires an aesthetic, modern Client-Side Rendered (CSR) web interface for `ui/web` and the `ui/tauri` desktop wrapper that excels at high-density dashboarding, data visualizations, and fast post-render interactions.
Evaluation of the frontend landscape against Rigix's principles (compile-time rigidity, offline-first reliability, and presentation-only shells) revealed:
1. **Leptos CSR (Rust / WASM)**: While matching the server's language, WebAssembly binaries have high cold-start latency, large payloads, complex wasm-bindgen tooling, and lack mature modern dashboard charting libraries.
2. **React 19**: While offering a large ecosystem, the Virtual DOM creates runtime re-render overhead when interacting with dense charts and data grids. Furthermore, React hooks suffer from stale closures and re-render loops that degrade AI-agent code generation.
3. **Headless UI Component Libraries (Ark UI, Kobalte)**: While providing prebuilt state machines, Ark UI and Kobalte are undergoing active migrations to bridge Solid 2.0's breaking reactive changes, introducing peer-dependency friction and third-party abstraction lock-in.

## Decision
1. **Framework Core**: Standardize strictly on **Solid 2.0 + TypeScript (Strict) + Vite** (powered by the Rust/Oxc compiler). Solid's fine-grained reactivity runs component setups once, eliminating the Virtual DOM entirely and providing 60fps post-render interaction performance.
2. **Headless Architecture & Zero-Dependency Primitives**: Avoid third-party headless UI libraries (Ark UI, Kobalte) in favor of **modern native HTML5 web platform standards** (`<dialog>` with `.showModal()`, native `popover="auto"` for tooltips/popovers/menus, `<details>`/`<summary>` for accordions) styled with **Tailwind CSS v4** design tokens. This achieves total immunity to Solid 2.0 ecosystem upgrade friction, zero black-box abstractions, and superior AI-assisted code generation.
3. **Hardware-Accelerated Visualization Engine**: Standardize on **Apache ECharts** (HTML5 Canvas / WebGL) fronted by Rigix-owned declarative, strictly-typed JSX components (`<SplineChart />`, `<BarChart />`, `<AreaChart />`) in `ui/web/src/components/charts/`. This enables 60fps fluid scrubbing, panning, and brushing over tens of thousands of data points without DOM thrashing.
4. **Data Grid & Offline State**:
   - Standardize on **TanStack Table for Solid** (`@tanstack/solid-table`) for compile-time verified tabular data grids.
   - Use **IndexedDB** for the local read-through response cache and the persistent FIFO mutation queue.
5. **Unified Static Build & Tauri Packaging**:
   - `ui/web` produces a single static SPA build output (`ui/web/dist`).
   - The compiled Axum server binary embeds these exact static files (via `rust-embed` or `include_dir`) for browser delivery.
   - Tauri 2.0 bundles this exact `dist/` directory locally (`frontendDist: "../ui/web/dist"`), ensuring the desktop app boots instantly and inspects local IndexedDB cached state even when disconnected from the server.
6. **Compile-Time Contract Rigidity**: Derive all TypeScript interfaces directly from the backend's `openapi.json` via `openapi-typescript`, enforcing strict compiler flags (`noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`).

## Consequences
- The web and desktop client shells achieve 60fps canvas charting and zero-re-render UI responsiveness.
- The project has zero external component library dependencies, ensuring 100% immediate compatibility with Solid 2.0 and full code ownership.
- Desktop and web targets share an identical static build pipeline with zero build divergence.
- Tauri desktop clients can boot and render from the local IndexedDB cache completely offline.
