# Rigix Domain Glossary

Rigix is a reusable Copier project template for bootstrapping self-hosted, multi-user applications with a server-authoritative Single Source of Truth and decoupled presentation-only client shells.

## Language

**Single Source of Truth (SSOT)**:
The authoritative PostgreSQL database and backend server runtime that owns all data schemas, validation, persistence, and state transitions.
_Avoid_: Client-side store, distributed truth, offline authority

**Auth Schema (`auth`)**:
The dedicated PostgreSQL schema containing Rigix baseline infrastructure (users, credentials, refresh-token rotation, session revocations).
_Avoid_: User table in public, core schema

**Domain Schema (`public`)**:
The PostgreSQL schema reserved entirely for downstream applications to define raw ingestion tables, transformation triggers, and analytical views.
_Avoid_: Template schema, global namespace

**Raw Layer (Bronze)**:
The ingestion tables that store incoming mutations and external data payloads in a 1:1, un-opinionated format before transformation.
_Avoid_: Staging cache, temporary tables

**Operational Layer (Silver)**:
The validated, deduplicated, and strongly-typed relational tables and views produced by database triggers and cleanups from the Raw Layer.
_Avoid_: Core domain, middle tier

**Analytical Layer (Gold)**:
The denormalized tables, materialized views, and joined views optimized for dashboard reads, aggregations, and metrics.
_Avoid_: Reporting database, data warehouse

**Materialized View Refresh Worker**:
The asynchronous background task in the server that refreshes gold analytical views concurrently without blocking client read requests.
_Avoid_: On-demand calculation, cron script

**Read-Through Cache**:
The client-side local database store that strictly mirrors server responses for immediate presentation rendering.
_Avoid_: Client database, offline write store

**Mutation Queue (FIFO Outbox)**:
The persistent client-side FIFO queue that buffers outgoing user actions while offline, dispatching them to the server upon reconnection without optimistic UI updates.
_Avoid_: Background sync, optimistic mutations

**Presentation Shell (`ui/*`)**:
A decoupled client application target that contains zero business logic, domain validation, or database authority, rendering exclusively from the local Read-Through Cache and dispatching user actions to the Mutation Queue.
_Avoid_: Smart client, fat client

**Embedded Web Client (`ui/web`)**:
The static Client-Side Rendered (CSR) Solid 2.0 application compiled to static HTML, CSS, and JavaScript, embedded directly into the Axum server binary and packaged into the desktop shell.
_Avoid_: SSR web app, dynamic web server

**Desktop Shell (`ui/tauri`)**:
The native desktop wrapper that packages the local embedded web client bundle and connects to the backend API via a runtime-configurable URL while supporting offline-first boot.
_Avoid_: Native desktop UI, remote-URL webview
