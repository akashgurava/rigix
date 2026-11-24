# 0005: Copier Template Questions and Conditional Target Directory Generation

## Context

Rigix is a domain-agnostic project template providing a compiled Rust backend server (`server/`) and up to five client presentation shells under `ui/` (`web`, `android`, `apple`, `flutter`, `tauri`).

When bootstrapping a new project via Copier, developers have diverse platform requirements. Forcing all five frontend skeletons onto every downstream project imposes substantial maintenance overhead, repository bloat, and heavy local SDK prerequisites (Android Studio/SDK, Xcode/macOS, Flutter SDK).

However, unstructured or excessive prompt questions create cognitive fatigue. Furthermore:
1. `server/` and `ui/web/` form the operational core of self-hosted homelab applications: the compiled Rust binary directly embeds the static CSR web bundle (`ui/web/dist`) to deliver an immediate, zero-ops web interface at `/`. Making either optional introduces unnecessary branching in the backend server and Docker build definitions.
2. `ui/tauri/` is a lightweight desktop wrapper around `ui/web/dist`. It is secondary to web and mobile targets and must be strictly opt-in.
3. Mobile targets require dedicated platform SDKs. Kotlin/Compose on Android (`ui/android`) is the preferred default mobile shell, while Apple (`ui/apple`) and Flutter (`ui/flutter`) are optional.
4. Downstream agent guidelines and documentation must remain strictly scoped to the active platforms without polluting unselected repositories with irrelevant architecture docs.

## Decision

1. **Mandatory Core Targets**:
   - `server/` and `ui/web/` are non-optional and always generated. No exclusion prompts are presented for them.
   - The Axum server statically embeds `ui/web/dist`, and the multi-stage `Dockerfile` always builds both `ui/web` and `server/`.

2. **Template Profile Presets (`ui_profile`)**:
   - `copier.yml` presents a top-level choice question `ui_profile`:
     - `default` (Recommended): Scaffolds `server` + `ui/web` + `ui/android`.
     - `minimal`: Scaffolds `server` + `ui/web` only (ideal for headless or pure web deployments).
     - `full`: Scaffolds all 5 presentation shells (`web`, `android`, `apple`, `flutter`, `tauri`).
     - `custom`: Prompts for individual optional targets (`include_android` [default: true], `include_tauri` [default: false], `include_apple` [default: false], `include_flutter` [default: false]).
   - Internal boolean evaluation variables are derived in `copier.yml`:
     - `_include_android`: `true` if `ui_profile in ['default', 'full']` or (`ui_profile == 'custom' and include_android`), else `false`.
     - `_include_tauri`: `true` if `ui_profile == 'full'` or (`ui_profile == 'custom' and include_tauri`), else `false`.
     - `_include_apple`: `true` if `ui_profile == 'full'` or (`ui_profile == 'custom' and include_apple`), else `false`.
     - `_include_flutter`: `true` if `ui_profile == 'full'` or (`ui_profile == 'custom' and include_flutter`), else `false`.

3. **Core Identity & Sanitized Namespace Variables**:
   - `project_name`: Slug format (alphanumeric, hyphens, underscores).
   - `project_title`: Human-readable name (default: `{{ project_name | replace('-', ' ') | replace('_', ' ') | title }}`).
   - `project_description`: Short summary tagline.
   - `app_id`: Reverse-DNS identifier (e.g. `com.example.{{ project_name | replace('_', '-') }}`).
   - `package_name`: Computed sanitized JVM identifier replacing hyphens with underscores (`{{ app_id | replace('-', '_') }}`) for Android Gradle and Kotlin namespaces.

4. **Conditional Directory Exclusion via `_exclude`**:
   - Exclusion is managed strictly through Jinja expressions in `copier.yml`'s `_exclude` list.
   - Source repository directory names remain standard without Jinja directory name mangling, ensuring full IDE, linter, and LSP compatibility during template authoring.
   - Exclusions cover both code trees (`ui/<target>/**`) and corresponding documentation (`docs/agents/targets/<target>.md`).

5. **Target-Grouped Documentation Architecture**:
   - General guidelines reside in root `AGENTS.md` and `GLOSSARY.md`.
   - Platform-specific instructions are modularized under `docs/agents/targets/<target>.md` and conditionally excluded when their target is disabled.

6. **Zero-Task Lifecycle**:
   - `_tasks: []` is left empty. Copier never runs automated package manager installs or git commands, guaranteeing fast, offline, and deterministic project generation.

7. **Three-Profile CI Test Matrix**:
   - Automated template test harness verifies the three canonical profiles: `minimal`, `default`, and `full`.

## Consequences

- Fast and intuitive project bootstrapping: most users select `default` or `minimal` in a single prompt.
- Clean downstream repositories with zero dead code or orphaned documentation for inactive platforms.
- Complete separation between template authoring and runtime generation without mangled folder names.
- Zero network dependencies during template copy.
