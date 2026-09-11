# Local candidate worker v1

This Python 3.12 standard-library tool asks the configured localhost Qwen model
for bounded text operations. It validates and stores a private candidate and
diff; it never writes the task worktree, executes model output, runs tests,
installs dependencies, merges, or publishes.

Create a host config outside the repository from `config.example.json`. On this
Mac the configured `ai_root` is `/Users/sunqi/ai`, so records are stored under
`/Users/sunqi/ai/outputs/local-worker/<job_id>/`.

```bash
export STAGEMASTER_LOCAL_WORKER_CONFIG=/Users/sunqi/ai/outputs/local-worker/config.json
python3 tools/local-worker/worker.py run --task /absolute/path/task.json
python3 tools/local-worker/worker.py status --job JOB_ID
python3 tools/local-worker/worker.py cancel --job JOB_ID
python3 tools/local-worker/worker.py repair --job JOB_ID --diagnostics /absolute/path/diagnostics.txt
python3 tools/local-worker/worker.py verify-source --job JOB_ID
```

`run` is foreground and prints the job identity immediately, then prints the
final recorded state. There is no queue or daemon. A process-wide file lock
makes a second generation return `busy`. Exact duplicate task input returns its
existing record; reusing a task ID with different input returns
`identity_conflict`.

Before a request, the worker requires an absolute, clean Git worktree whose HEAD
equals the task's full `base_commit`. Existing write files must also be readable.
Lists are capped at six paths. Absolute paths, traversal, symlink components,
binary files, protected/write overlap, unknown JSON fields, and oversized input
are rejected. The model sees only task text and listed files.

The only accepted tool call is `propose_changes`, containing `create` and exact
`replace` operations. Replacements must have a non-empty match occurring exactly
once in the in-memory candidate. All operations validate before any artifact is
published, and the source is rehashed before candidate construction. Candidate
files, readable diff, source/candidate/proposal hashes, raw responses, timing,
upstream usage, model identity, and errors are preserved per attempt.

`candidate_ready` means text validation passed. Sol must run `verify-source`,
copy or apply only the candidate paths into the task worktree, review the diff,
format the accepted files, run the separately stored acceptance commands, and
record the real result. If tests fail, Sol removes only its accepted task changes
after checking source hashes, saves diagnostics, and calls `repair` explicitly.
The worker never consumes repair attempts automatically; total recorded model
time is capped by the task.

Cancellation is cooperative: it records `cancel_requested`, waits for the
bounded non-streaming HTTP request, saves and discards a late response, then
records `cancelled`. This does not claim the shared MLX server stopped early.
Timeout, disconnect, or process loss records backend state as `unknown` and
blocks later generation. After independently verifying the runtime, Sol can use
the explicit recovery command:

```bash
python3 tools/local-worker/worker.py recover --job JOB_ID --backend-state finished
```

This path restriction protects what the worker itself reads and emits. It is
not an OS sandbox; Sol executes generated code later with the normal Codex host
permissions. Current verification covers macOS only.

Run the synthetic suite:

```bash
python3 -m unittest discover -s tools/local-worker/tests -v
```
