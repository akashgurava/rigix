# Target Agent Guidelines: Web Client (`ui/web`)

## Framework & Toolchain
- **Stack**: Solid 2.0, TypeScript (Strict), Vite with Oxc compiler, Tailwind CSS v4.
- **Contract**: Derive all DTO interfaces directly from backend `openapi.json` via `openapi-typescript`.
- **Status**: **Mandatory Core Target** (compiled static CSR assets embedded directly in Axum server binary).

## Persistence & State
- **Storage**: IndexedDB via `idb`.
- **Offline Mirror**: `read_cache` table (unparsed raw JSON envelopes).
- **FIFO Outbox**: `mutation_outbox` table with 8 standard columns.
- **Concurrency**: Native Web Locks API (`navigator.locks.request('rigix_outbox_drain', ...)`).

## UI Primitives & Visualizations
- **Primitives**: Zero third-party component libraries. Use native HTML5 `<dialog>` and `popover="auto"`.
- **Charts**: Declarative typed JSX wrappers over Apache ECharts (HTML5 Canvas/WebGL).
