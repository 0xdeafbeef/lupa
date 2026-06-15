#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


def metric(row, name):
    if name == "commands":
        return len(row["commands"])
    if name == "uncached":
        return row["uncached_input_tokens"]
    if name in ["input", "cached", "output"]:
        key = {
            "input": "input_tokens",
            "cached": "cached_input_tokens",
            "output": "output_tokens",
        }[name]
        return row["usage"].get(key, 0)
    if name == "reasoning":
        return row["usage"].get(
            "reasoning_output_tokens",
            row["usage"].get("output_tokens_details", {}).get("reasoning_tokens", 0),
        )
    if name in ["lupa_cmd", "rg_cmd"]:
        return row["tool_hits"][name]
    if name == "words":
        final_json = row.get("final_json")
        if final_json is None:
            try:
                final_json = json.loads(row.get("final_text", "{}"))
            except json.JSONDecodeError:
                final_json = {}
        return len(final_json.get("answer", "").split())
    return row[name]


def pct_delta(value, baseline):
    if baseline == 0:
        return None
    return round((value - baseline) / baseline * 100, 1)


def relative_line(field, value, baseline):
    pct = pct_delta(value, baseline)
    delta = round(value - baseline, 3)
    if pct is None:
        return f"{field}: {value} vs baseline {baseline}"
    if delta == 0:
        label = "same"
    elif field == "wall_seconds":
        label = "faster" if delta < 0 else "slower"
    else:
        label = "less" if delta < 0 else "more"
    return f"{field}: {abs(pct):.1f}% {label} ({value} vs baseline {baseline}, delta={delta})"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("summary")
    args = parser.parse_args()

    summary = Path(args.summary)
    rows = json.loads(summary.read_text())
    bad = [row for row in rows if row.get("ok") is False or row.get("exit_code", 0) != 0]
    if bad:
        print("invalid runs")
        for row in bad:
            print(f"{row.get('task')} {row.get('arm')}: {', '.join(row.get('errors', []))}")
        raise SystemExit(1)

    fields = ["wall_seconds", "input", "cached", "uncached", "output", "reasoning", "commands", "lupa_cmd", "rg_cmd", "words"]
    print("per-run")
    for row in rows:
        print("\t".join([row["task"], row["arm"]] + [f"{field}={metric(row, field)}" for field in fields]))

    print("\narm totals")
    totals_by_arm = {}
    for arm in sorted({row["arm"] for row in rows}):
        arm_rows = [row for row in rows if row["arm"] == arm]
        totals = {field: round(sum(metric(row, field) for row in arm_rows), 3) for field in fields[:-1]}
        totals_by_arm[arm] = totals
        print(arm, json.dumps(totals, sort_keys=True))

    with_lupa = totals_by_arm.get("with_lupa")
    without_lupa = totals_by_arm.get("without_lupa")
    if with_lupa and without_lupa:
        print("\nwith_lupa relative to without_lupa baseline")
        for field in ["wall_seconds", "input", "uncached", "output", "commands"]:
            print(relative_line(field, with_lupa[field], without_lupa[field]))

    print("\npairs")
    by_task = {}
    for row in rows:
        by_task.setdefault(row["task"], {})[row["arm"]] = row
    for task, pair in by_task.items():
        with_lupa = pair.get("with_lupa")
        without_lupa = pair.get("without_lupa")
        if not with_lupa or not without_lupa:
            continue
        deltas = {
            field: {
                "delta": round(metric(with_lupa, field) - metric(without_lupa, field), 3),
                "pct": pct_delta(metric(with_lupa, field), metric(without_lupa, field)),
            }
            for field in ["wall_seconds", "input", "uncached", "output", "commands"]
        }
        print(task, json.dumps(deltas, sort_keys=True))


if __name__ == "__main__":
    main()
