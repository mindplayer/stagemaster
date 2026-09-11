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

Every created attempt reaches a recorded terminal state when context or request
preparation is rejected; no-request failures keep the backend `finished` and do
not block the next valid task. Complete and partial HTTP response bytes are saved
before JSON or proposal parsing, elapsed request time always consumes the task
budget, and unavailable usage is recorded as `null` rather than estimated.

`candidate_ready` means text validation passed. Sol must run `verify-source`,
copy or apply only the candidate paths into the task worktree, review the diff,
format the accepted files, run the separately stored acceptance commands, and
record the real result. If tests fail, Sol removes only its accepted task changes
after checking source hashes, saves diagnostics, and calls `repair` explicitly.
The worker never consumes repair attempts automatically; total recorded model
time is capped by the task.

Cancellation and candidate publication use the same per-job locked transition.
An accepted cancellation before publication prevents a new candidate from
becoming eligible; a cancellation arriving after terminal publication returns
that terminal state without changing its flag. A response that arrives after an
accepted cancellation is saved for evidence and discarded. This does not claim
the shared MLX server stopped early.

One monotonic deadline covers connection, response headers, and success or error
body reads. It is the smaller of the per-request limit and remaining task budget.
At the deadline a watchdog closes the same HTTP connection and is joined; the
worker does not kill the shared model server or leave a helper HTTP request.
Timeout, disconnect, truncated response, or byte-limit closure records backend
state as `unknown` and blocks later generation. After independently verifying
the runtime, Sol can use the explicit recovery command:

```bash
python3 tools/local-worker/worker.py recover --job JOB_ID --backend-state finished
```

This path restriction protects what the worker itself reads and emits. It is
not an OS sandbox; Sol executes generated code later with the normal Codex host
permissions. Current verification covers macOS only.

Run the synthetic and loopback HTTP suite (33 tests at v0.1.1):

```bash
python3 -m unittest discover -s tools/local-worker/tests -v
```
