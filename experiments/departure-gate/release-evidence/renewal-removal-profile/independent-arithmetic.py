"""Recompute finite profiling arithmetic from preserved stdout; never launch tools.

Usage: python independent-arithmetic.py CAPTURE --output NEW_JSON [--summary DRIVER_JSON]
All reads are local. The sole write is an exclusive new JSON file. Historical
timings never enter the calculations. This is not a benchmark or readiness gate.
"""
from pathlib import Path
import argparse
import datetime
import hashlib
import json
import statistics


PHASES = ("growth", "growth_last_10_percent", "steady", "unrelated_changes",
          "failed_release", "release")
PATHS = ("renewal_removal", "readings")
FIELDS = ("blocks", "searches", "removes", "matches", "misses", "probes",
          "shifted_elements", "estimated_shifted_header_bytes",
          "shared_cow_detaches", "cow_cloned_histories", "cow_cloned_occurrences",
          "block_ns", "search_ns", "cow_ns", "remove_ns")
STATE_FIELDS = ("save_sha256", "serialized_save_bytes", "retired_dynamic_records",
                "withdrawals", "undrained")
ARCHIVE_FIELDS = ("ndjson_sha256", "ndjson_bytes", "records", "provenance_nodes",
                  "growth_ndjson_bytes", "growth_records", "growth_nodes",
                  "max_undrained_items", "items_at_growth_end")
WORK_FIELDS = ("dirty_owners", "references_scanned", "vertices_visited",
               "edges_visited", "candidates_examined", "collected")
RATIOS = ("block_share_of_apply", "block_share_of_departure", "cow_share_of_block",
          "search_share_of_block", "remove_share_of_block")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def ratio(numerator, denominator):
    return numerator / denominator if denominator else None


def med(values):
    numbers = [n for n in values if n is not None]
    return statistics.median(numbers) if numbers else None


def spread(values):
    numbers = [n for n in values if n is not None]
    return dict(count=len(numbers), min=min(numbers) if numbers else None,
                median=med(numbers), max=max(numbers) if numbers else None)


def semantic_projection(data):
    return {
        **{key: data[key] for key in ("source_sha256", "cycles", "mode", "drain_every")},
        **{state: {key: data[state][key] for key in STATE_FIELDS}
           for state in ("before_release", "after_release")},
        "archive": {key: data["archive"][key] for key in ARCHIVE_FIELDS},
        "phases": {phase: {"samples": data[phase]["samples"],
                  **({"collector_work": data[phase]["collector_work"]}
                     if data[phase]["samples"] and phase != "failed_release" else {})}
                   for phase in PHASES},
    }


def validate_profile(values, label):
    for field in FIELDS:
        require(type(values[field]) is int and values[field] >= 0, f"{label}.{field}")
    require(values["blocks"] == values["searches"], f"{label}: blocks/searches")
    require(values["matches"] + values["misses"] == values["searches"], f"{label}: search partition")
    require(values["removes"] == values["matches"], f"{label}: matches/removes")
    require(values["shared_cow_detaches"] <= values["removes"], f"{label}: detach count")
    require(values["probes"] >= values["matches"], f"{label}: probe count")
    require(all(values[field] <= values["block_ns"] for field in ("search_ns", "cow_ns", "remove_ns")),
            f"{label}: nested timer exceeds block")


def sample_projection(sample):
    apply_ns = sample["ns"]
    departure_ns = sample["removal"]["departure_ns"]
    require(type(apply_ns) is int and apply_ns >= 0, "invalid apply duration")
    require(type(departure_ns) is int and 0 <= departure_ns <= apply_ns, "invalid departure duration")
    result = dict(apply_ns=apply_ns, departure_ns=departure_ns,
                  peak_additional_bytes=sample["peak_additional_bytes"], work=sample["work"])
    for kind in PATHS:
        p = sample["removal"][kind]
        validate_profile(p, kind)
        require(p["block_ns"] <= departure_ns, "block exceeds departure")
        result[kind] = {**p, "block_share_of_apply": ratio(p["block_ns"], apply_ns),
                        "block_share_of_departure": ratio(p["block_ns"], departure_ns),
                        "cow_share_of_block": ratio(p["cow_ns"], p["block_ns"]),
                        "search_share_of_block": ratio(p["search_ns"], p["block_ns"]),
                        "remove_share_of_block": ratio(p["remove_ns"], p["block_ns"])}
    return result


def phase_projection(plain, diagnostic, label):
    require(plain["samples"] == diagnostic["samples"], f"{label}: samples differ")
    n = plain["samples"]
    if not n:
        return {"samples": 0}
    profile = diagnostic["removal"]
    for kind in PATHS:
        validate_profile(profile[kind], f"{label}.{kind}")
    raw = profile["small_phase_samples"]
    require((raw is None and n > 3) or (isinstance(raw, list) and len(raw) == n and n <= 3),
            f"{label}: unexpected raw sample population")
    samples = [sample_projection(sample) for sample in raw or []]
    if raw:
        times = sorted(sample["ns"] for sample in raw)
        require(diagnostic["median_ns"] == times[n // 2], f"{label}: native median")
        require(diagnostic["mean_ns"] == sum(times) // n, f"{label}: native mean")
        require(diagnostic["max_ns"] == times[-1], f"{label}: native max")
        require(diagnostic["p99_ns"] == times[(n * 99 + 99) // 100 - 1], f"{label}: native p99")
        require(diagnostic["max_dispatch_additional_heap_bytes"] == max(s["peak_additional_bytes"] for s in raw),
                f"{label}: peak from raw samples")
        require(profile["departure_ns"] == sum(s["removal"]["departure_ns"] for s in raw),
                f"{label}: departure sum")
        for kind in PATHS:
            for field in FIELDS:
                require(profile[kind][field] == sum(s["removal"][kind][field] for s in raw),
                        f"{label}.{kind}.{field}: total disagrees with raw samples")
        for field in WORK_FIELDS:
            require(diagnostic["collector_work"][field] == sum(s["work"][field] for s in raw),
                    f"{label}.{field}: work total disagrees with raw samples")
        require(diagnostic["collector_work"]["max_work_items"] == max(s["work"]["peak_work_items"] for s in raw),
                f"{label}: max work items")
    return {
        "samples": n,
        "plain_median_ns": plain["median_ns"], "diagnostic_median_ns": diagnostic["median_ns"],
        "diagnostic_to_plain_latency_ratio": ratio(diagnostic["median_ns"], plain["median_ns"]),
        "plain_peak_bytes": plain["max_dispatch_additional_heap_bytes"],
        "diagnostic_peak_bytes": diagnostic["max_dispatch_additional_heap_bytes"],
        "peak_delta_bytes": diagnostic["max_dispatch_additional_heap_bytes"] - plain["max_dispatch_additional_heap_bytes"],
        "attempted_removal_totals": {kind: profile[kind] for kind in PATHS},
        "departure_ns_total": profile["departure_ns"], "raw_small_phase_samples": samples,
        "process_sample_medians": {
            kind: {field: med(s[kind][field] for s in samples) for field in (*FIELDS, *RATIOS)} for kind in PATHS},
        "plain_work": plain["collector_work"], "diagnostic_work": diagnostic["collector_work"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--summary", type=Path, help="Optional post-computation cross-check of driver summary")
    args = parser.parse_args()
    root = args.capture.resolve()
    require(not args.output.exists(), "Refuse to overwrite analysis")
    manifest = {}

    def payload(relative, expected=None):
        file = (root / relative).resolve()
        require(file.is_relative_to(root), f"Escaping capture path: {relative}")
        data = file.read_bytes()
        value = digest(data)
        require(expected is None or expected == value, f"Hash mismatch: {relative}")
        manifest[relative] = value
        return data

    def read(relative, expected=None):
        return json.loads(payload(relative, expected).decode("utf-8"))

    contract = read("contract.json")
    contract_hash = manifest["contract.json"]
    registration = read("registration.json")
    require(registration["contractSha256"] == contract_hash, "Contract changed")
    require(registration["driverSha256"] == contract["driverSha256"], "Registered driver mismatch")
    require(contract["freshProcesses"] == 26 and contract["pairs"] == 13 and len(contract["matrix"]) == 13,
            "Unexpected registered population")
    for item in contract["fixtures"] + contract["referenceManifest"]:
        payload(item["file"], item["sha256"])
    payload("collector_profile.rs", contract["baseline"]["harnessSha256"])
    identities = {kind: read(f"{kind}.json") for kind in ("plain", "diagnostic")}
    for kind, identity in identities.items():
        require(identity["contractSha256"] == contract_hash, "Identity contract mismatch")
        payload(identity["file"], identity["sha256"])
        observation = read(identity["buildObservation"]["file"], identity["buildObservation"]["sha256"])
        require(observation["status"] == 0 and observation["error"] is None and observation["inputsStable"] is True,
                f"Unsuccessful {kind} build")
        require(observation["revision"] == observation["headAfter"] == identity["revision"] and not observation["trackedDiffAfter"],
                f"Dirty/wrong {kind} build")
        require(observation["argv"] == contract["builds"][kind] and observation["inputs"] == identity["inputs"],
                f"{kind} build contract mismatch")
        require(observation["executableSha256"] == identity["sha256"], "Build/executable mismatch")
        for item in identity["buildStreams"]:
            payload(item["file"], item["sha256"])
    require(identities["plain"]["revision"] == identities["diagnostic"]["revision"], "Build head differs")
    require(identities["plain"]["inputs"] == identities["diagnostic"]["inputs"], "Build source differs")
    attempts = [json.loads(line) for line in payload("attempts.jsonl").decode("utf-8").splitlines() if line]
    expected_order = [(f'{c["name"]}-{c["cycles"]}-t{c["trial"]}', v)
                      for c in contract["matrix"] for v in c["order"]]
    require([(a["key"], a["variant"]) for a in attempts] == expected_order, "Attempt population/order mismatch")
    rows = []
    for cell in contract["matrix"]:
        key = f'{cell["name"]}-{cell["cycles"]}-t{cell["trial"]}'
        fixture = next(f for f in contract["fixtures"] if f["name"] == cell["name"])
        captured = {}
        for variant in ("plain", "diagnostic"):
            row = read(f"capture/{key}-{variant}.json")
            attempt = next(a for a in attempts if a["key"] == key and a["variant"] == variant)
            require(row["status"] == 0 and row["signal"] is None and row["error"] is None and not row.get("parseError"),
                    f"Unsuccessful capture: {key}/{variant}")
            for field in ("command", "startedAt", "revision", "executableSha256", "contractSha256"):
                require(row[field] == attempt[field], f"Ledger mismatch {key}/{variant}/{field}")
            identity = identities[variant]
            require(row["revision"] == identity["revision"] and row["executableSha256"] == identity["sha256"] and row["contractSha256"] == contract_hash,
                    f"Artifact identity mismatch: {key}/{variant}")
            require(row["fixtureSha256"] == fixture["sha256"], "Fixture mismatch")
            require(row["command"][2:] == [str(cell["cycles"]), fixture["mode"], "1", "100"], "Arguments differ")
            data = read(row["stdout"]["file"], row["stdout"]["sha256"])
            payload(row["stderr"]["file"], row["stderr"]["sha256"])
            require(data == row["data"], "Metadata disagrees with raw stdout")
            require(data["instrumented"] is True, "Collector counters missing")
            require(data["schema"] == ("caveat-collector-native-profile/1" if variant == "plain" else "caveat-renewal-removal-diagnostic/1"), "Schema differs")
            if variant == "diagnostic":
                require(data["diagnostic_only"] is True, "Diagnostic identity missing")
            captured[variant] = data
        p, d = captured["plain"], captured["diagnostic"]
        reference = next(r for r in contract["references"] if r["key"] == key)
        historical = read(reference["file"], reference["sha256"])
        old_data = read("accepted-reference/" + historical["stdout"]["file"], historical["stdout"]["sha256"])
        require(semantic_projection(p) == semantic_projection(d) == semantic_projection(old_data), f"Semantic inequality: {key}")
        phases = {phase: phase_projection(p[phase], d[phase], f"{key}/{phase}") for phase in PHASES}
        memory = {state: dict(plain=p[state]["retained_rust_heap_bytes"], diagnostic=d[state]["retained_rust_heap_bytes"],
                             delta=d[state]["retained_rust_heap_bytes"] - p[state]["retained_rust_heap_bytes"])
                  for state in ("before_release", "after_release")}
        rows.append({**cell, "key": key, "phases": phases, "retained_heap_bytes": memory,
                     "max_heap_freed_by_drain": {"plain": p["archive"]["max_heap_freed_by_drain"], "diagnostic": d["archive"]["max_heap_freed_by_drain"]},
                     "semantic_projection": semantic_projection(p)})
    primary = [row for row in rows if row["primary"]]
    require(len(primary) == 4, "Primary population differs")
    aggregate = {}
    for phase in ("release", "failed_release"):
        values = [r["phases"][phase] for r in primary]
        aggregate[phase] = {
            **{field: spread(v[field] for v in values) for field in
               ("plain_median_ns", "diagnostic_median_ns", "diagnostic_to_plain_latency_ratio", "plain_peak_bytes", "diagnostic_peak_bytes", "peak_delta_bytes")},
            **{kind: {field: spread(v["process_sample_medians"][kind][field] for v in values)
                      for field in (*FIELDS, *RATIOS)} for kind in PATHS},
        }
    scaling = []
    for row in sorted((r for r in rows if r["name"] == "release-mutual"), key=lambda r: (r["cycles"], r["trial"])):
        fields = row["phases"]["release"]["process_sample_medians"]["renewal_removal"]
        scaling.append({"key": row["key"], "cycles": row["cycles"], "trial": row["trial"],
                        "probes": fields["probes"], "shifted_elements": fields["shifted_elements"],
                        "probes_per_cycle_squared": ratio(fields["probes"], row["cycles"] ** 2),
                        "shifts_per_cycle_squared": ratio(fields["shifted_elements"], row["cycles"] ** 2),
                        "probes_per_remove": ratio(fields["probes"], fields["removes"]),
                        "shifts_per_remove": ratio(fields["shifted_elements"], fields["removes"]),
                        "cow_cloned_occurrences": fields["cow_cloned_occurrences"],
                        "block_ns": fields["block_ns"], "block_share_of_apply": fields["block_share_of_apply"]})
    summary_check = None
    if args.summary:
        summary_bytes = args.summary.read_bytes()
        summary = json.loads(summary_bytes.decode("utf-8"))
        require(summary["contractSha256"] == contract_hash and summary["complete"] is True, "Summary identity differs")
        require(len(summary["rows"]) == len(rows), "Summary population differs")
        for computed, reported in zip(rows, summary["rows"]):
            require(all(computed[k] == reported[k] for k in ("name", "cycles", "trial", "order", "primary")), "Summary order differs")
            require(computed["retained_heap_bytes"] == reported["retained_heap_bytes"], "Summary retained heap differs")
            for phase in PHASES:
                a, b = computed["phases"][phase], reported["phases"][phase]
                for field in ("samples", "plain_median_ns", "diagnostic_median_ns", "diagnostic_to_plain_latency_ratio",
                              "plain_peak_bytes", "diagnostic_peak_bytes", "peak_delta_bytes", "attempted_removal_totals", "departure_ns_total"):
                    require(a.get(field) == b.get(field), f"Summary differs: {computed['key']}/{phase}/{field}")
                if a["samples"]:
                    for kind in PATHS:
                        for field in ("block_ns", "block_share_of_apply", "block_share_of_departure", "cow_ns"):
                            require(a["process_sample_medians"][kind][field] == b["process_sample_medians"][kind][field], "Summary process median differs")
        for phase in ("release", "failed_release"):
            for field in ("plain_median_ns", "diagnostic_median_ns", "diagnostic_to_plain_latency_ratio"):
                require(aggregate[phase][field] == summary["primarySummary"][phase][field], "Summary primary distribution differs")
            for kind in PATHS:
                for field in ("block_ns", "block_share_of_apply", "block_share_of_departure", "cow_ns"):
                    require(aggregate[phase][kind][field] == summary["primarySummary"][phase][kind][field], "Summary primary attribution differs")
        summary_check = dict(file=str(args.summary.resolve()), sha256=digest(summary_bytes), arithmetic_agrees=True)
    result = dict(schema="caveat-renewal-independent-arithmetic/1", recorded_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  capture=str(root), script_sha256=digest(Path(__file__).read_bytes()), contract_sha256=contract_hash,
                  revision=identities["plain"]["revision"], native_executables={k: v["sha256"] for k, v in identities.items()},
                  fresh_processes=26, pairs=13, finite_semantic_equality=True, primary=aggregate,
                  mutual_scaling=scaling, rows=rows, driver_summary_crosscheck=summary_check,
                  read_manifest=[dict(file=k, sha256=v) for k, v in sorted(manifest.items())],
                  limits=["No commands, native runs, source edits, performance target or optimization verdict.",
                          "Plain native stdout contains phase summaries only; its individual apply samples cannot be independently reconstructed.",
                          "Diagnostic raw samples exist only for phases of three or fewer events; their medians, peaks, sums and ratios are independently recomputed.",
                          "Large-phase attribution totals remain native-recorded totals; no aggregate time is divided by a single median.",
                          "Growth tail overlaps growth; never add those populations. Four process medians remain four observations, not twelve independent failures.",
                          "Historical accepted stdout is used for semantic fields only, never timings or memory comparisons.",
                          "Counters describe this renewal corpus; zero readings do not measure reading-stream cost. COW costs exclude earlier map detachment.",
                          "Nested clocks and diagnostic inventory perturb apply time; shares do not predict a replacement's speedup.",
                          "Heap is requested Rust bytes, not RSS. Preallocated diagnostic samples are excluded from retained-session measurements.",
                          "This arithmetic replay validates saved observations and hashes, not global host isolation or universal behavior."])
    with args.output.open("x", encoding="utf-8", newline="\n") as file:
        json.dump(result, file, indent=2, ensure_ascii=False, allow_nan=False)
        file.write("\n")
    print(json.dumps(dict(output=str(args.output.resolve()), output_sha256=digest(args.output.read_bytes()),
                          processes=26, pairs=13, primary=aggregate, summary_crosscheck=summary_check), indent=2))


if __name__ == "__main__":
    main()
