# Web & Desktop Presentation Shell Specification

## Problem Statement

Users interacting with self-hosted analytical applications expect responsive, aesthetically modern dashboards with high-density data visualizations and instant post-render interactions. However, traditional web dashboard frameworks frequently suffer from Virtual DOM re-render stutter, large payload bundles, and fragile third-party component library upgrade churn. Furthermore, desktop wrappers often introduce divergent packaging pipelines or fail to boot when disconnected from the backend.

## Solution

Rigix provides a pure-presentation Client-Side Rendered (CSR) Solid 2.0 application paired with Tailwind CSS v4, Apache ECharts, and TanStack Table. The interface uses zero third-party component libraries by relying on native modern HTML5 platform standards (`<dialog>`, `popover="auto"`), compile-time contract derivation from `openapi.json`, and an identical static export shared between the server's embedded web route and Tauri 2.0 desktop shell.

## User Stories

1. As a dashboard viewer, I want fine-grained UI reactivity without Virtual DOM re-render pauses, so that complex dashboards with dozens of widgets remain smooth and responsive.
2. As a data analyst, I want hardware-accelerated interactive charts that handle tens of thousands of data points at 60fps, so that I can pan, zoom, and brush time-series metrics without browser lag.
3. As a dashboard viewer, I want tabular data grids with type-safe sorting, filtering, and pagination, so that I can inspect structured records cleanly.
4. As an application developer, I want all frontend API models derived directly from the backend's OpenAPI contract, so that API contract mismatches are caught at compile time.
5. As a desktop user, I want a native desktop application powered by Tauri 2.0 that boots instantly and loads cached data even when disconnected from the server.
6. As an application operator, I want the web client embedded directly in the compiled server binary, so that deploying the application requires running only a single container without a separate frontend web server.
7. As an application developer, I want modal dialogs and dropdown menus built on native HTML5 primitives (`<dialog>` and `popover`), so that the project avoids breaking changes from third-party headless UI packages.
8. As a user across varied devices, I want an interface styled with Tailwind CSS v4 design tokens, so that the presentation is consistent and optimized for dark-mode data density.
9. As an offline desktop or web user, I want the client to render existing data from a local IndexedDB cache, so that disconnected boots never show blank screens.
10. As an offline user, I want mutations to enqueue safely in a persistent FIFO queue without optimistic UI drift, so that the client presentation strictly reflects server truth.
11. As a frontend developer, I want fast Hot Module Replacement (HMR) powered by Vite and the Oxc compiler, so that development iteration cycles remain instantaneous.
12. As a desktop user, I want the Tauri desktop application to support runtime-configurable server URLs, so that I can easily point the shell at local or remote backend instances.

## Implementation Decisions

- **Framework Core**: Standardized strictly on Solid 2.0 + TypeScript (Strict) + Vite (Oxc compiler). Solid's fine-grained reactivity executes component setups once on mount, eliminating the Virtual DOM and providing immediate post-render interaction performance.
- **Zero-Dependency Native Primitives**: Standardize exclusively on modern native HTML5 web platform standards styled with Tailwind CSS v4 design tokens:
  - Modals leverage native `<dialog>` with `.showModal()`.
  - Dropdowns, menus, and tooltips leverage native `popover="auto"`.
  - Accordions leverage native `<details>` and `<summary>`.
- **Hardware-Accelerated Visualization Engine**: Standardized on Apache ECharts (HTML5 Canvas / WebGL) fronted by Rigix-owned declarative JSX components (`<SplineChart />`, `<BarChart />`, `<AreaChart />`) managing canvas lifecycles and resize observers.
- **Tabular Data Grid**: TanStack Table for Solid (`@tanstack/solid-table`) for compile-time verified tabular data views.
- **Unified Static Build & Tauri Packaging**:
  - `ui/web` produces a single static SPA build output (`dist/`).
  - The compiled Axum server binary embeds these exact static files for browser delivery.
  - Tauri 2.0 bundles this exact `dist/` directory locally from `ui/tauri/src-tauri/` (`frontendDist: "../../web/dist"`), ensuring identical runtime behavior across web and desktop shells.
- **Compile-Time Contract Rigidity**: Derive all TypeScript interfaces directly from the backend's `openapi.json` via `openapi-typescript` and `openapi-fetch`, enforcing strict compiler flags (`noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`).
- **Styling Architecture**: Tailwind CSS v4 with custom dark-mode design tokens tailored for high-density analytical dashboards.

## Testing Decisions

- **Good Test Definition**: Good frontend tests test user-observable interactions against rendered DOM elements (e.g. verifying dialog visibility upon click, chart canvas container mounting, table sorting triggers), rather than inspecting signal internals or mocking reactive graphs.
- **Modules Tested**:
  - Modal dialog: verifies open/close state synchronization with native `<dialog>`.
  - Native popover: verifies toggle behavior and focus containment.
  - Chart wrapper: verifies mount, resize observer observation, and cleanup disposal.
  - Data table: verifies column rendering, sorting, and pagination controls.
- **Prior Art**: Vitest with `@solidjs/testing-library` and DOM testing utilities.

## Out of Scope

- Server-Side Rendering (SSR) or dynamic Node.js runtime hosting.
- React, Leptos, or WebAssembly (WASM) frontend implementations.
- Client-side business logic, domain calculations, or offline authority.
- Third-party headless UI component dependencies.

## Further Notes

- Implements the architectural decisions recorded in `docs/adr/0002-solid-web-csr-and-tauri.md`.
