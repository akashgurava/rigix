# Copier Template Configuration & Conditional Directory Generation Specification

## 1. Overview

This specification establishes the exact configuration for `copier.yml`, defining project identity variables, prompt questions, preset profiles, Jinja exclusion rules, and target-grouped documentation layout for Rigix template generation.

---

## 2. Target Matrix & Profiles

Rigix defines a server-authoritative core with decoupled presentation shells:

| Component | Target Path | Inclusion Status | Default in `default` | Default in `minimal` | Default in `full` |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Backend Server** | `server/` | **Mandatory** | Yes | Yes | Yes |
| **Embedded Web Client** | `ui/web/` | **Mandatory** | Yes | Yes | Yes |
| **Android Presentation Shell** | `ui/android/` | Optional | **Yes** | No | Yes |
| **Desktop Shell** | `ui/tauri/` | Optional | No | No | Yes |
| **Apple Presentation Shell** | `ui/apple/` | Optional | No | No | Yes |
| **Flutter Presentation Shell** | `ui/flutter/` | Optional | No | No | Yes |

---

## 3. `copier.yml` Specification

```yaml
# ==============================================================================
# Rigix Copier Template Configuration
# ==============================================================================

_min_copier_version: "9.0.0"

# ------------------------------------------------------------------------------
# 1. Project Identity Prompts
# ------------------------------------------------------------------------------

project_name:
  type: str
  help: "Project repository slug (lowercase alphanumeric, hyphens, and underscores)"
  default: "rigix_app"
  validator: >-
    {% if not (project_name | regex_search('^[a-zA-Z0-9_-]+$')) %}
    project_name must only contain letters, numbers, hyphens, or underscores.
    {% endif %}

project_title:
  type: str
  help: "Human-readable application title"
  default: "{{ project_name | replace('-', ' ') | replace('_', ' ') | title }}"

project_description:
  type: str
  help: "Short project summary and tagline"
  default: "Self-hosted, offline-first application generated with Rigix"

app_id:
  type: str
  help: "Reverse-DNS application identifier (used by Android, Apple, and Tauri)"
  default: "com.example.{{ project_name | replace('_', '-') }}"
  validator: >-
    {% if not (app_id | regex_search('^[a-zA-Z0-9_]+(\\.[a-zA-Z0-9_-]+)+$')) %}
    app_id must follow reverse-DNS notation (e.g. com.example.my-app).
    {% endif %}

# Computed JVM-safe package name for Android Kotlin namespace & directories
package_name:
  type: str
  default: "{{ app_id | replace('-', '_') }}"

# ------------------------------------------------------------------------------
# 2. UI Presentation Shell Profile Selection
# ------------------------------------------------------------------------------

ui_profile:
  type: str
  help: "Select UI presentation shell profile"
  choices:
    - "default: Web + Android (Recommended)"
    - "minimal: Web only (Lightweight / Headless)"
    - "full: All 5 targets (Web, Android, Apple, Flutter, Tauri)"
    - "custom: Choose platforms individually"
  default: "default: Web + Android (Recommended)"

# Custom Platform Toggles (only prompted when ui_profile == 'custom')
include_android:
  type: bool
  help: "Include Android (Kotlin / Jetpack Compose / Room) shell?"
  default: true
  when: "{{ ui_profile.startswith('custom') }}"

include_tauri:
  type: bool
  help: "Include Tauri 2.0 desktop shell wrapping web client?"
  default: false
  when: "{{ ui_profile.startswith('custom') }}"

include_apple:
  type: bool
  help: "Include Apple (Swift / SwiftUI / GRDB) shell?"
  default: false
  when: "{{ ui_profile.startswith('custom') }}"

include_flutter:
  type: bool
  help: "Include Flutter (Dart / Drift) shell?"
  default: false
  when: "{{ ui_profile.startswith('custom') }}"

# ------------------------------------------------------------------------------
# 3. Computed Target Flags
# ------------------------------------------------------------------------------

_include_android:
  type: bool
  default: >-
    {%- if ui_profile.startswith('default') -%}
      true
    {%- elif ui_profile.startswith('full') -%}
      true
    {%- elif ui_profile.startswith('minimal') -%}
      false
    {%- else -%}
      {{ include_android }}
    {%- endif -%}

_include_tauri:
  type: bool
  default: >-
    {%- if ui_profile.startswith('full') -%}
      true
    {%- elif ui_profile.startswith('custom') -%}
      {{ include_tauri }}
    {%- else -%}
      false
    {%- endif -%}

_include_apple:
  type: bool
  default: >-
    {%- if ui_profile.startswith('full') -%}
      true
    {%- elif ui_profile.startswith('custom') -%}
      {{ include_apple }}
    {%- else -%}
      false
    {%- endif -%}

_include_flutter:
  type: bool
  default: >-
    {%- if ui_profile.startswith('full') -%}
      true
    {%- elif ui_profile.startswith('custom') -%}
      {{ include_flutter }}
    {%- else -%}
      false
    {%- endif -%}

# ------------------------------------------------------------------------------
# 4. Conditional Directory and Documentation Exclusions
# ------------------------------------------------------------------------------

_exclude:
  - ".git"
  - ".DS_Store"
  - "copier.yml"
  # Android code and target docs
  - "{% if not _include_android %}ui/android/**{% endif %}"
  - "{% if not _include_android %}docs/agents/targets/android.md{% endif %}"
  # Tauri code and target docs
  - "{% if not _include_tauri %}ui/tauri/**{% endif %}"
  - "{% if not _include_tauri %}docs/agents/targets/tauri.md{% endif %}"
  # Apple code and target docs
  - "{% if not _include_apple %}ui/apple/**{% endif %}"
  - "{% if not _include_apple %}docs/agents/targets/apple.md{% endif %}"
  # Flutter code and target docs
  - "{% if not _include_flutter %}ui/flutter/**{% endif %}"
  - "{% if not _include_flutter %}docs/agents/targets/flutter.md{% endif %}"

# ------------------------------------------------------------------------------
# 5. Lifecycle Hooks
# ------------------------------------------------------------------------------

# Keep tasks empty for deterministic, offline, and instant template instantiation
_tasks: []
```

---

## 4. Target-Grouped Documentation Architecture

Target-specific instructions and agent operating protocols are organized in `docs/agents/targets/`:

```
docs/agents/
├── domain.md
├── issue-tracker.md
├── triage-labels.md
└── targets/
    ├── web.md        # Mandatory (Solid 2.0, Vite, Tailwind v4, idb, Canvas ECharts)
    ├── android.md    # Conditional: _include_android (Compose, Room, Flow, Outbox)
    ├── tauri.md      # Conditional: _include_tauri (Tauri 2.0, Webview bundle, IPC)
    ├── apple.md      # Conditional: _include_apple (SwiftUI, GRDB.swift, Actor worker)
    └── flutter.md    # Conditional: _include_flutter (Flutter, Drift, SQLite)
```

The root `AGENTS.md` and `GLOSSARY.md` files maintain cross-platform, domain-wide standards (PostgreSQL 18 SSOT, Head-of-Line blocking, Quiescence sync, TDD), linking to `docs/agents/targets/<target>.md` for platform-specific details.

---

## 5. Template Verification Smoke-Test Matrix

CI tests validate template generation across the three canonical profiles:

1. **Profile: `minimal`**:
   - Command: `copier copy --data 'project_name=smoke_minimal' --data 'ui_profile=minimal: Web only (Lightweight / Headless)' . /tmp/smoke_minimal`
   - Verification:
     - `server/` and `ui/web/` exist.
     - `docs/agents/targets/web.md` exists.
     - `ui/android`, `ui/tauri`, `ui/apple`, `ui/flutter` do not exist.
     - Platform docs for android, tauri, apple, flutter do not exist.

2. **Profile: `default`**:
   - Command: `copier copy --data 'project_name=smoke_default' --data 'ui_profile=default: Web + Android (Recommended)' . /tmp/smoke_default`
   - Verification:
     - `server/`, `ui/web/`, and `ui/android/` exist.
     - `docs/agents/targets/web.md` and `docs/agents/targets/android.md` exist.
     - `ui/tauri`, `ui/apple`, `ui/flutter` do not exist.

3. **Profile: `full`**:
   - Command: `copier copy --data 'project_name=smoke_full' --data 'ui_profile=full: All 5 targets (Web, Android, Apple, Flutter, Tauri)' . /tmp/smoke_full`
   - Verification:
     - All 5 `ui/*` directories exist.
     - All 5 `docs/agents/targets/*.md` exist.
