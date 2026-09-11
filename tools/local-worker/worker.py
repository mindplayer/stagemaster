#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

from local_worker import __version__
from local_worker.errors import WorkerError
from local_worker.runner import Runner
from local_worker.task import load_config


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="stagemaster-local-worker")
    parser.add_argument(
        "--config",
        type=Path,
        default=Path(os.environ["STAGEMASTER_LOCAL_WORKER_CONFIG"])
        if "STAGEMASTER_LOCAL_WORKER_CONFIG" in os.environ
        else None,
        help="trusted host config (or STAGEMASTER_LOCAL_WORKER_CONFIG)",
    )
    parser.add_argument("--version", action="version", version=__version__)
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("run")
    run.add_argument("--task", type=Path, required=True)
    status = commands.add_parser("status")
    status.add_argument("--job", required=True)
    cancel = commands.add_parser("cancel")
    cancel.add_argument("--job", required=True)
    repair = commands.add_parser("repair")
    repair.add_argument("--job", required=True)
    repair.add_argument("--diagnostics", type=Path, required=True)
    recover = commands.add_parser("recover")
    recover.add_argument("--job")
    recover.add_argument("--backend-state", choices=["finished", "still_running", "unknown"], required=True)
    verify = commands.add_parser("verify-source")
    verify.add_argument("--job", required=True)
    return parser


def emit(value: dict, *, error: bool = False) -> None:
    print(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True), file=sys.stderr if error else sys.stdout, flush=True)


def main() -> int:
    arguments = build_parser().parse_args()
    if arguments.config is None:
        emit(
            {"error": {"class": "missing_config", "message": "pass --config or set STAGEMASTER_LOCAL_WORKER_CONFIG", "details": {}}},
            error=True,
        )
        return 2
    try:
        runner = Runner(load_config(arguments.config))
        started = lambda status: emit({"job_id": status["job_id"], "state": "running"})
        if arguments.command == "run":
            result = runner.run(arguments.task, on_started=started)
        elif arguments.command == "status":
            result = runner.store.load(arguments.job)
        elif arguments.command == "cancel":
            result = runner.cancel(arguments.job)
        elif arguments.command == "repair":
            result = runner.repair(arguments.job, arguments.diagnostics, on_started=started)
        elif arguments.command == "recover":
            result = runner.recover(arguments.job, arguments.backend_state)
        elif arguments.command == "verify-source":
            result = runner.verify_candidate_source(arguments.job)
        else:
            raise WorkerError("invalid_command", f"unsupported command: {arguments.command}")
        emit(result)
        return 0
    except WorkerError as exc:
        emit({"error": exc.as_dict()}, error=True)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
