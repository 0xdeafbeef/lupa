#!/usr/bin/env python3
import argparse
import concurrent.futures
import json
import re
import shlex
import subprocess
import time
from pathlib import Path


ARMS = {
    "with_lupa": (
        "Use `lupa` for code exploration before full source reads. Start with "
        "`lupa map` on the named file(s), then use `lupa show`/`lupa keys` for "
        "selected symbols. Do not run `lupa digest`; the relevant files are named."
    ),
    "without_lupa": (
        "Do not invoke the local source-navigation binary at all. Use `rg` and "
        "short targeted file reads instead."
    ),
}

CODEX_REPO_WORKERS = 8

SUITES = {
    "lupa": {
        "label": "local checkout of the `lupa` repository",
        "out": Path("/tmp/lupa-codex-eval-targeted"),
        "answer_words": 180,
        "tasks": [
            (
                "show_flow",
                "Using only src/cli.rs, src/render.rs, src/model.rs, and "
                "src/adapters/mod.rs, explain the runtime flow for `lupa show` "
                "from CLI dispatch through source rendering.",
            ),
            (
                "stdin_contract",
                "Using only src/cli.rs, src/model.rs, and tests/cli_v1.rs, "
                "determine the exact current contract for stdin language mode: "
                "accepted commands, token recognition, and invalid multi-arg behavior.",
            ),
            (
                "relaxed_matching",
                "Using only src/render.rs and tests/cli_v1.rs, explain relaxed "
                "`lupa show` key matching and identify which tests protect "
                "exact-match priority and ambiguity.",
            ),
            (
                "digest_behavior",
                "Using only src/cli.rs, src/walk.rs, src/adapters/mod.rs, "
                "src/render.rs, and tests/cli_v1.rs, explain how `lupa digest` "
                "chooses supported files, skips paths, and renders compact "
                "per-file summaries.",
            ),
        ],
    },
    "codex": {
        "label": "local checkout of the Codex repository",
        "out": Path("/tmp/lupa-codex-eval-codex-repo"),
        "answer_words": 220,
        "tasks": [
            (
                "cli_feature_toggle_flow",
                "Using only codex-rs/cli/src/main.rs, codex-rs/features/src/lib.rs, "
                "codex-rs/core/src/config/managed_features.rs, and "
                "codex-rs/cli/tests/features.rs, explain the flow for feature "
                "toggles from CLI parsing through config override or `codex "
                "features enable/disable` persistence. Include how unknown or "
                "legacy feature names are handled.",
            ),
            (
                "web_search_feature_flow",
                "Using only codex-rs/features/src/lib.rs, "
                "codex-rs/features/src/legacy.rs, codex-rs/core/src/config/mod.rs, "
                "codex-rs/core/src/config/config_tests.rs, and "
                "codex-rs/core/tests/suite/web_search.rs, explain how web search "
                "mode is resolved from top-level config and legacy feature flags, "
                "including per-turn permission-profile fallback.",
            ),
            (
                "multi_agent_v2_config_flow",
                "Using only codex-rs/features/src/lib.rs, codex-rs/core/src/config/mod.rs, "
                "codex-rs/core/src/config/config_tests.rs, and "
                "codex-rs/core/src/session/config_lock.rs, explain how "
                "features.multi_agent_v2 is loaded, validated, and materialized "
                "into runtime config or config-lock output.",
            ),
            (
                "managed_feature_requirements_flow",
                "Using only codex-rs/core/src/config/managed_features.rs, "
                "codex-rs/features/src/lib.rs, codex-rs/features/src/legacy.rs, "
                "and codex-rs/core/src/config/config_tests.rs, explain how "
                "configured feature values are normalized against feature requirements, "
                "how pinned values are enforced, and how warnings are produced for "
                "legacy or unknown requirement keys.",
            ),
        ],
    },
}


def allowed_files(question):
    return set(re.findall(r"[A-Za-z0-9_./-]+\.(?:rs|py|md|toml)", question))


def inspected_command(command):
    try:
        parts = shlex.split(command)
    except ValueError:
        return command
    if len(parts) >= 3 and Path(parts[0]).name in ["bash", "sh"] and parts[1] in ["-c", "-lc"]:
        return parts[2]
    return command


def command_count(commands, name):
    total = 0
    for command in commands:
        try:
            parts = shlex.split(inspected_command(command))
        except ValueError:
            parts = inspected_command(command).split()
        total += sum(1 for part in parts if Path(part).name == name)
    return total


def command_files(commands):
    files = set()
    for command in commands:
        for word in re.findall(r"[A-Za-z0-9_./-]+\.(?:rs|py|md|toml)", inspected_command(command)):
            files.add(word)
    return files


def final_status(exit_code, final_text, hits, arm, files, command_total, inspected_files):
    errors = []
    if exit_code != 0:
        errors.append(f"exit_code={exit_code}")
    try:
        parsed = json.loads(final_text)
    except json.JSONDecodeError:
        errors.append("final_text is not JSON")
        parsed = {}
    evidence_files = parsed.get("evidence_files", [])
    if isinstance(evidence_files, list):
        for path in evidence_files:
            if path not in files:
                errors.append(f"unexpected evidence_file={path}")
    else:
        errors.append("evidence_files is not a list")
    for path in inspected_files:
        if path not in files:
            errors.append(f"unexpected inspected_file={path}")
    for key in ["answer", "evidence_files", "commands_used", "confidence"]:
        if not parsed.get(key):
            errors.append(f"missing {key}")
    if command_total > 8:
        errors.append(f"too many commands={command_total}")
    for name in ["sed", "nl", "awk"]:
        if hits[f"{name}_cmd"]:
            errors.append(f"used {name}")
    if arm == "without_lupa" and hits["lupa_cmd"]:
        errors.append("without_lupa used lupa")
    if arm == "with_lupa" and not hits["lupa_cmd"]:
        errors.append("with_lupa did not use lupa")
    return parsed, errors


def run_one(repo, out, suite, task, arm):
    task_id, question = task
    common = f"""\
You are evaluating a {suite['label']}.
This is a read-only task. Do not edit files. Do not use web search.
Use shell commands only to inspect local files.
The task names the only files you may inspect.
Keep exploration tight: at most 8 shell commands.
Do not collect exact line citations. Do not run nl, sed, or awk.
It is enough to identify repo-relative files that support the answer.
Final response must be valid JSON only, with this object shape:
{{
  "answer": "concise technical answer, max {suite['answer_words']} words",
  "evidence_files": ["repo/path", "..."],
  "commands_used": ["command summary", "..."],
  "confidence": "high|medium|low"
}}
"""
    prompt = "\n\n".join([common, f"Tool policy: {ARMS[arm]}", f"Task: {question}"])
    started = time.monotonic()
    proc = subprocess.run(
        [
            "codex",
            "-C",
            str(repo),
            "-s",
            "read-only",
            "-a",
            "never",
            "exec",
            "--ignore-rules",
            "--json",
            "-o",
            str(out / f"{task_id}.{arm}.last.json"),
            prompt,
        ],
        cwd=repo,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=900,
    )
    wall = time.monotonic() - started
    (out / f"{task_id}.{arm}.jsonl").write_text(proc.stdout)

    usage = {}
    commands = []
    final_text = ""
    thread_id = None
    for line in proc.stdout.splitlines():
        if not line.startswith("{"):
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("type") == "thread.started":
            thread_id = event.get("thread_id")
        elif event.get("type") == "turn.completed":
            usage = event.get("usage", {})
        item = event.get("item") or {}
        if item.get("type") == "agent_message" and item.get("text"):
            final_text = item["text"]
        if item.get("type") == "command_execution" and item.get("status") == "completed":
            commands.append(item.get("command", ""))

    hits = {
        "lupa_cmd": command_count(commands, "lupa"),
        "rg_cmd": command_count(commands, "rg"),
        "sed_cmd": command_count(commands, "sed"),
        "nl_cmd": command_count(commands, "nl"),
        "awk_cmd": command_count(commands, "awk"),
    }
    final_json, errors = final_status(
        proc.returncode,
        final_text,
        hits,
        arm,
        allowed_files(question),
        len(commands),
        command_files(commands),
    )
    return {
        "task": task_id,
        "arm": arm,
        "ok": not errors,
        "errors": errors,
        "exit_code": proc.returncode,
        "wall_seconds": round(wall, 3),
        "usage": usage,
        "uncached_input_tokens": usage.get("input_tokens", 0) - usage.get("cached_input_tokens", 0),
        "thread_id": thread_id,
        "tool_hits": hits,
        "commands": commands,
        "final_text": final_text,
        "final_json": final_json,
    }


def failed_result(task_id, arm, err):
    return {
        "task": task_id,
        "arm": arm,
        "ok": False,
        "errors": [str(err)],
        "exit_code": None,
        "wall_seconds": 0,
        "usage": {},
        "uncached_input_tokens": 0,
        "thread_id": None,
        "tool_hits": {"lupa_cmd": 0, "rg_cmd": 0, "sed_cmd": 0, "nl_cmd": 0, "awk_cmd": 0},
        "commands": [],
        "final_text": "",
        "final_json": {},
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("suite", choices=SUITES)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    args = parser.parse_args()

    suite = SUITES[args.suite]
    repo = args.repo.resolve()
    out = suite["out"]
    out.mkdir(parents=True, exist_ok=True)
    jobs = [(task, arm) for task in suite["tasks"] for arm in ARMS]

    results = []
    if args.suite == "codex":
        with concurrent.futures.ThreadPoolExecutor(max_workers=CODEX_REPO_WORKERS) as executor:
            futures = {}
            for task, arm in jobs:
                print(f"running {task[0]} {arm}", flush=True)
                futures[executor.submit(run_one, repo, out, suite, task, arm)] = (task[0], arm)
            for future in concurrent.futures.as_completed(futures):
                task_id, arm = futures[future]
                try:
                    result = future.result()
                except Exception as err:
                    result = failed_result(task_id, arm, err)
                print(json.dumps(result, ensure_ascii=False), flush=True)
                results.append(result)
    else:
        for task, arm in jobs:
            print(f"running {task[0]} {arm}", flush=True)
            result = run_one(repo, out, suite, task, arm)
            print(json.dumps(result, ensure_ascii=False), flush=True)
            results.append(result)

    summary_path = out / "summary.json"
    summary_path.write_text(json.dumps(results, indent=2, ensure_ascii=False))
    print(f"summary {summary_path}", flush=True)


if __name__ == "__main__":
    main()
