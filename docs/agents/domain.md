# Domain Docs

How autonomous agents consume Rigix domain documentation when exploring the codebase.

## Before exploring, read these

- **`GLOSSARY.md`** at the repo root: Defines project-specific vocabulary, core concepts, and explicit anti-synonyms (`_Avoid_`).
- **`docs/adr/`**: Read ADRs that touch the domain area you are working in (e.g. storage, UI, or local persistence).

## File structure

Rigix is a single-context repository:

```
/
├── GLOSSARY.md
├── docs/
│   ├── adr/
│   │   ├── 0001-postgresql-storage-and-pipeline.md
│   │   ├── 0002-solid-web-csr-and-tauri.md
│   │   └── 0003-client-local-storage-and-outbox.md
│   ├── agents/
│   │   ├── domain.md
│   │   ├── issue-tracker.md
│   │   └── triage-labels.md
│   ├── specs/
│   │   ├── client-storage.md
│   │   ├── storage.md
│   │   └── web-ui.md
│   └── project-intentions.md
```

## Use the glossary's vocabulary

When naming domain concepts in tasks, proposals, tests, or specifications, adhere strictly to the vocabulary established in `GLOSSARY.md`. If a required concept is absent, record it for addition during domain modeling.

## Flag ADR conflicts

If proposed work contradicts an existing ADR in `docs/adr/`, surface it explicitly rather than silently overriding.
