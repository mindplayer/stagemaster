# Local candidate worker v1 — retired

Retired from the StageMaster workflow on 2026-09-11. Sol develops directly and
Astra coordinates architecture and reviews; see
[DEV-ADR-001](../../docs/development/decisions/DEV-ADR-001-sol-astra.md).
The remaining review defects are unresolved. Do not run or maintain this worker
for current development tasks; the CLI has not been technically disabled.

The implementation and tests remain here as historical source. Trial projects,
raw requests, responses, candidate code, diagnostics, and the original host
configuration have moved into this project:

- [Worker records](../../data/development/legacy-qwen/local-worker)
- [Original Rust trial](../../data/development/legacy-qwen/trial-20260911)
- [Location policy and verified migration](../../docs/development/project-files.md)

Archived records and scripts retain their original bytes, including historical
absolute paths. They are evidence, not live configuration or startup commands.
Use the documented path mapping to inspect them, and do not recreate the old
shared-AI output directory.

The legacy `ai_root` schema field is retained in `config.example.json` solely
for source reference; its example now points inside a project-owned data root.
It is not permission to restart the retired workflow.
