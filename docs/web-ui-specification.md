# Rigix Web & Desktop Shell Specification

**Target Framework:** Solid 2.0 + TypeScript (Strict) + Vite (Oxc compiler)  
**Desktop Shell:** Tauri 2.0 (bundled static web assets)  
**Styling System:** Tailwind CSS v4 (bespoke data-dense design tokens)  
**Visualization Engine:** Apache ECharts (Canvas/WebGL) with declarative typed JSX wrappers  
**Design Philosophy:** Server-Authoritative SSOT, Zero VDOM, Pure Presentation Shell, Zero Component Library Lock-in.

---

## 1. Core Principles

1. **Pure Presentation Shell**: The web client contains zero business rules, domain calculations, or client-side authority. It renders exclusively from the local Read-Through Cache and dispatches user actions to the FIFO Mutation Queue.
2. **Zero Virtual DOM & Signal Reactivity**: Solid 2.0 executes component setups once on mount. State transitions update only the subscribing DOM text node or canvas buffer, ensuring instantaneous post-render interactions.
3. **Compile-Time Rigidity**: All API request/response models are generated directly from the backend's `openapi.json` via `openapi-typescript`. The TypeScript compiler runs with `"strict": true`, `"noUncheckedIndexedAccess": true`, and `"exactOptionalPropertyTypes": true`.
4. **Zero-Dependency Native Primitives**: Instead of depending on fragile or migrating third-party headless UI packages (Ark UI, Kobalte), UI controls leverage native modern HTML5 elements (`<dialog>`, `popover="auto"`, `<details>`) styled with Tailwind CSS v4.
5. **Unified Static Delivery**: A single `ui/web/dist` artifact serves both the Axum server's embedded static file route (`rust-embed`) and Tauri 2.0's local desktop shell (`frontendDist`).

---

## 2. Technology Stack & Roles

| Role | Technology | Justification |
| :--- | :--- | :--- |
| **Framework Core** | **Solid 2.0** (`solid-js@^2.0.0`) | Fine-grained signals without Virtual DOM; async primitives in reactive graph; native TypeScript `.tsx` checked by standard `tsc`. |
| **Build & Toolchain** | **Vite** + **Oxc Compiler** | High-speed Rust-based compiler toolchain; instant HMR; single-command static export (`dist/`). |
| **Styling** | **Tailwind CSS v4** (`@tailwindcss/vite`) | Utility-first CSS with zero runtime; custom design tokens tailored for high-density, dark-mode-first analytical dashboards. |
| **Data Visualizations** | **Apache ECharts** + Typed Wrappers | Hardware-accelerated HTML5 Canvas/WebGL rendering; handles 50k+ data points at 60fps with native data-zoom sliders and multi-series brushing. |
| **Tabular Data Grid** | **TanStack Table for Solid** | 100% compile-time type-safe column definitions, sorting, filtering, and pagination. |
| **Offline Cache & Outbox** | **IndexedDB** (via `idb`) | Asynchronous, structured persistence for read-through response caching and FIFO mutation queuing without main-thread blocking. |
| **Contract Client** | **`openapi-fetch`** + **`openapi-typescript`** | Zero-runtime-bloat client derived from `server/`'s `openapi.json`. |
| **Desktop Wrapper** | **Tauri 2.0** | Native WebView wrapper embedding `ui/web/dist` locally for offline-first boot and OS window integration. |

---

## 3. Component Architecture: Zero-Dependency Native Primitives

All UI components live in `ui/web/src/components/ui/` and utilize modern HTML5 standards:

### 3.1 Modal Dialog (`<dialog>`)
```tsx
import { type JSX, onCleanup } from "solid-js";

interface ModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly title: string;
  readonly children: JSX.Element;
}

export function Modal(props: ModalProps) {
  let dialogRef: HTMLDialogElement | undefined;

  // Sync open state with native dialog C++ API
  createEffect(() => {
    if (!dialogRef) return;
    if (props.isOpen && !dialogRef.open) {
      dialogRef.showModal();
    } else if (!props.isOpen && dialogRef.open) {
      dialogRef.close();
    }
  });

  return (
    <dialog
      ref={dialogRef}
      onClose={() => props.onClose()}
      class="rounded-xl bg-zinc-900 border border-zinc-800 p-6 text-zinc-100 shadow-2xl backdrop:bg-black/60 max-w-lg w-full"
    >
      <div class="flex items-center justify-between pb-3 border-b border-zinc-800">
        <h3 class="font-semibold text-lg">{props.title}</h3>
        <button
          onClick={() => props.onClose()}
          class="text-zinc-400 hover:text-zinc-200 text-sm px-2 py-1"
        >
          ✕
        </button>
      </div>
      <div class="mt-4">{props.children}</div>
    </dialog>
  );
}
```

### 3.2 Native Popover (`popover="auto"`)
```tsx
import type { JSX } from "solid-js";

interface PopoverProps {
  readonly id: string;
  readonly triggerText: string;
  readonly children: JSX.Element;
}

export function Popover(props: PopoverProps) {
  return (
    <div class="relative inline-block">
      <button
        popovertarget={props.id}
        class="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-100 text-sm font-medium border border-zinc-700"
      >
        {props.triggerText}
      </button>
      <div
        id={props.id}
        popover="auto"
        class="p-4 rounded-xl bg-zinc-900 border border-zinc-800 text-zinc-100 shadow-2xl m-2"
      >
        {props.children}
      </div>
    </div>
  );
}
```

---

## 4. Hardware-Accelerated Declarative Charting

All charts wrap Apache ECharts inside `ui/web/src/components/charts/`. The component sets up canvas buffers once in `onMount` and tears them down in `onCleanup`.

### 4.1 Typed Spline Chart (`<SplineChart />`)
```tsx
import { onMount, onCleanup, createEffect } from "solid-js";
import * as echarts from "echarts/core";
import { LineChart } from "echarts/charts";
import { GridComponent, TooltipComponent, DataZoomComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";

echarts.use([LineChart, GridComponent, TooltipComponent, DataZoomComponent, CanvasRenderer]);

export interface SplineChartProps<T extends Record<string, unknown>> {
  readonly data: readonly T[];
  readonly xKey: keyof T;
  readonly series: readonly {
    readonly key: keyof T;
    readonly label: string;
    readonly color: string;
  }[];
  readonly dataZoom?: boolean;
}

export function SplineChart<T extends Record<string, unknown>>(props: SplineChartProps<T>) {
  let chartContainer: HTMLDivElement | undefined;
  let chartInstance: echarts.ECharts | undefined;

  onMount(() => {
    if (!chartContainer) return;
    chartInstance = echarts.init(chartContainer, "dark");

    const resizeObserver = new ResizeObserver(() => chartInstance?.resize());
    resizeObserver.observe(chartContainer);

    onCleanup(() => {
      resizeObserver.disconnect();
      chartInstance?.dispose();
    });
  });

  createEffect(() => {
    if (!chartInstance) return;

    chartInstance.setOption({
      backgroundColor: "transparent",
      tooltip: { trigger: "axis" },
      grid: { left: 40, right: 20, top: 20, bottom: props.dataZoom ? 60 : 30 },
      xAxis: {
        type: "category",
        data: props.data.map((d) => String(d[props.xKey])),
        axisLine: { lineStyle: { color: "#3f3f46" } },
      },
      yAxis: {
        type: "value",
        splitLine: { lineStyle: { color: "#27272a" } },
      },
      dataZoom: props.dataZoom ? [{ type: "slider", bottom: 10 }] : [],
      series: props.series.map((s) => ({
        name: s.label,
        type: "line",
        smooth: true,
        data: props.data.map((d) => Number(d[s.key])),
        itemStyle: { color: s.color },
      })),
    });
  });

  return <div ref={chartContainer} class="w-full h-80" />;
}
```

---

## 5. Offline-First Storage & Tauri 2.0 Integration

### 5.1 IndexedDB Cache & FIFO Outbox Schema
- **Read Cache Store (`read_cache`)**:
  - Key: Normalized endpoint URL + query string (e.g. `/api/v1/metrics?range=24h`).
  - Value: `{ payload: unknown, updatedAt: number, etag?: string }`.
  - Behavior: Views render immediately from cached records, revalidating in background when online.
- **Mutation Outbox Store (`mutation_outbox`)**:
  - Key: Auto-incrementing integer (FIFO ordering).
  - Value: `{ id: number, endpoint: string, method: string, payload: unknown, idempotencyKey: string, createdAt: number }`.
  - Behavior: Dispatches strictly sequentially upon network restoration; zero optimistic UI state modifications.

### 5.2 Tauri 2.0 Configuration (`ui/tauri/src-tauri/tauri.conf.json`)
```json
{
  "build": {
    "beforeDevCommand": "npm --prefix ../web run dev",
    "beforeBuildCommand": "npm --prefix ../web run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../web/dist"
  },
  "app": {
    "windows": [
      {
        "title": "{{ project_name }}",
        "width": 1280,
        "height": 800,
        "resizable": true
      }
    ]
  }
}
```
By packaging `../web/dist` directly into the Tauri app bundle, the desktop shell boots immediately into the local Solid 2.0 client and loads cached data from IndexedDB even if the host server is completely offline or stopped.
