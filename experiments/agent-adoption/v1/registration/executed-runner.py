#!/usr/bin/env python3
"""Prepare a frozen adoption-study packet, then run its registered trials.

Preparation never starts a Codex agent. Runtime records live below test-results/.
The packet boundary is procedural; this runner does not claim OS isolation.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import threading
import time

STUDY = Path(__file__).resolve().parent
ROOT = STUDY.parents[2]
SCHEMA = "caveat-adoption-registration/1"
TRIAL_IDS = ("A01", "A02", "A03")
TRIAL_SECONDS, TRIAL_BYTES = 1200, 32 * 1024 * 1024
EVALUATOR_SECONDS, EVALUATOR_BYTES = 120, 16 * 1024 * 1024
PRINT_LOCK = threading.Lock()


def now():
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_record(file):
    data = Path(file).read_bytes()
    return {"bytes": len(data), "sha256": digest(data)}


def json_read(file):
    return json.loads(Path(file).read_text(encoding="utf-8-sig"))


def json_write(file, value, exclusive=False):
    with Path(file).open("x" if exclusive else "w", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write("\n")


def status(event, **details):
    with PRINT_LOCK:
        print(json.dumps({"event": event, "at": now(), **details}), flush=True)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def tree_manifest(directory, exclude_node_modules=False):
    result = {}
    def traversal_error(error):
        raise error
    for current, directories, files in os.walk(directory, followlinks=False, onerror=traversal_error):
        current = Path(current)
        for name in list(directories):
            file = current / name
            if (exclude_node_modules and name == "node_modules") or name == "__pycache__":
                directories.remove(name)
            elif file.is_symlink():
                result[file.relative_to(directory).as_posix()] = {"symlink": os.readlink(file)}
                directories.remove(name)
        for name in sorted(files):
            file = current / name
            relative = file.relative_to(directory).as_posix()
            result[relative] = {"symlink": os.readlink(file)} if file.is_symlink() else file_record(file)
    return dict(sorted(result.items()))


def study_manifest():
    names = ["PROTOCOL.md", "TASK.md", "verify.mjs", "checks.test.mjs", "run-trials.py"]
    names += [file.relative_to(STUDY).as_posix() for file in sorted((STUDY / "evaluator").rglob("*")) if file.is_file()]
    return {name: file_record(STUDY / name) for name in names}


def child_environment():
    # Keep host authentication and executable discovery available without logging
    # values. Remove enclosing Codex conversation/context overrides explicitly.
    excluded = sorted(key for key in os.environ if key in {
        "CODEX_THREAD_ID", "CODEX_SESSION_ID", "CODEX_CONVERSATION_ID"})
    env = {key: value for key, value in os.environ.items() if key not in excluded}
    return env, excluded


def kill_owned_tree(process):
    if os.name == "nt":
        taskkill = Path(os.environ.get("SystemRoot", r"C:\Windows")) / "System32" / "taskkill.exe"
        result = subprocess.run([str(taskkill), "/PID", str(process.pid), "/T", "/F"],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=15,
                                creationflags=subprocess.CREATE_NO_WINDOW, shell=False)
        return {"method": "taskkill-owned-pid-tree", "pid": process.pid, "exitCode": result.returncode}
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    return {"method": "kill-owned-process-group", "pid": process.pid}


def bounded(command, cwd, directory, label, seconds, byte_limit, stdin=b"", env=None, cancel=None):
    """Stream raw output to disk, cap combined retained bytes and own the child."""
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=True)
    stdout_path, stderr_path = directory / f"{label}.stdout", directory / f"{label}.stderr"
    receipt_path = directory / f"{label}.receipt.json"
    require(not any(file.exists() for file in [stdout_path, stderr_path, receipt_path]), f"Refusing to overwrite {label} receipts")
    receipt = {"command": command, "cwd": str(cwd), "startedAt": now(), "timeoutSeconds": seconds,
               "combinedOutputLimitBytes": byte_limit, "exitCode": None, "stopReason": None}
    process = None
    output_limit = threading.Event()
    lock = threading.Lock()
    retained = 0
    observed = 0
    readers = []
    streams = []
    started = time.monotonic()
    try:
        streams = [stdout_path.open("xb"), stderr_path.open("xb")]
        status("process-starting", label=label, cwd=str(cwd))
        options = {"creationflags": subprocess.CREATE_NO_WINDOW | subprocess.CREATE_NEW_PROCESS_GROUP} if os.name == "nt" else {"start_new_session": True}
        process = subprocess.Popen(command, cwd=str(cwd), env=env, stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=False, **options)
        receipt["pid"] = process.pid
        status("process-started", label=label, pid=process.pid)

        def drain(pipe, output):
            nonlocal retained, observed
            try:
                while chunk := pipe.read1(8192):
                    with lock:
                        observed += len(chunk)
                        keep = min(len(chunk), max(0, byte_limit - retained))
                        retained += keep
                        if observed > byte_limit:
                            output_limit.set()
                    if keep:
                        output.write(chunk[:keep])
                        output.flush()
            except (OSError, ValueError):
                pass
            finally:
                pipe.close()

        for pipe, output in zip([process.stdout, process.stderr], streams):
            reader = threading.Thread(target=drain, args=(pipe, output), daemon=True)
            reader.start()
            readers.append(reader)

        def feed():
            try:
                process.stdin.write(stdin)
                process.stdin.flush()
            except (BrokenPipeError, OSError):
                pass
            finally:
                process.stdin.close()

        writer = threading.Thread(target=feed, daemon=True)
        writer.start()
        while process.poll() is None or any(reader.is_alive() for reader in readers):
            if cancel is not None and cancel.is_set():
                receipt["stopReason"] = "runner-interrupted"
            elif output_limit.is_set():
                receipt["stopReason"] = "output-limit"
            elif time.monotonic() - started >= seconds:
                receipt["stopReason"] = "timeout"
            if receipt["stopReason"]:
                receipt["termination"] = kill_owned_tree(process)
                break
            time.sleep(0.1)
        try:
            receipt["exitCode"] = process.wait(timeout=15)
        except subprocess.TimeoutExpired:
            process.kill()
            receipt["exitCode"] = process.wait(timeout=5)
            receipt["terminationFallback"] = "direct-owned-process-kill"
        for reader in readers:
            reader.join(timeout=5)
        writer.join(timeout=1)
        require(not any(reader.is_alive() for reader in readers), "Owned child output pipes remained open after termination")
        if output_limit.is_set() and receipt["stopReason"] is None:
            receipt["stopReason"] = "output-limit"
    except BaseException as error:
        receipt["error"] = f"{type(error).__name__}: {error}"
        if process is not None and process.poll() is None:
            receipt["termination"] = kill_owned_tree(process)
            receipt["exitCode"] = process.wait(timeout=15)
        if isinstance(error, KeyboardInterrupt) and cancel is not None:
            cancel.set()
    finally:
        for stream in streams:
            stream.close()
        receipt.update(finishedAt=now(), elapsedSeconds=round(time.monotonic() - started, 3),
                       retainedOutputBytes=retained, observedOutputBytes=observed)
        for name, file in [("stdout", stdout_path), ("stderr", stderr_path)]:
            if file.exists():
                receipt[name] = {"file": file.name, **file_record(file)}
        json_write(receipt_path, receipt, exclusive=True)
        status("process-finished", label=label, exitCode=receipt["exitCode"], stopReason=receipt["stopReason"], receipt=str(receipt_path))
    return receipt


def command_passed(receipt):
    return receipt["exitCode"] == 0 and receipt["stopReason"] is None and "error" not in receipt


def npm_command(node):
    directory = Path(node).resolve().parent
    candidates = [os.environ.get("npm_execpath"), directory / "node_modules/npm/bin/npm-cli.js",
                  directory.parent / "lib/node_modules/npm/bin/npm-cli.js", Path("/usr/share/nodejs/npm/bin/npm-cli.js")]
    for file in candidates:
        if file and Path(file).is_file() and Path(file).name == "npm-cli.js":
            return [str(node), str(Path(file).resolve())]
    raise ValueError("Cannot locate npm-cli.js beside Node; run with npm_execpath naming npm-cli.js")


def codex_command(executable, trial):
    return [str(executable), "exec", "--ephemeral", "--ignore-user-config", "--skip-git-repo-check", "--approve-for-me",
            "-c", 'windows.sandbox="elevated"', "-c", "project_doc_max_bytes=0", "-c", 'web_search="disabled"',
            "--json", "-C", str(trial), "-o", str(trial / "final-response.md"), "-"]


def preflight_header(data):
    """Read only the delimited human CLI header, never infer JSON metadata."""
    fields = {}
    separators = 0
    for line in data.decode("utf-8-sig", errors="replace").splitlines():
        if line.strip() == "--------":
            separators += 1
            if separators == 2:
                break
            continue
        if separators == 1 and ":" in line:
            key, value = line.split(":", 1)
            if key in {"model", "reasoning effort", "provider", "approval", "sandbox"}:
                fields[key] = value.strip()
    return fields


def prepare(args):
    package_report = Path(args.package_report).resolve()
    security_report = Path(args.security_report).resolve()
    package, security = json_read(package_report), json_read(security_report)
    require(package.get("schema") == 1 and package.get("name") == "caveat-lang", "Unsupported package report")
    require(security.get("schema") == "caveat-package-security/0.1" and security.get("passed") is True, "Security gate must pass")
    for scope in ["artifact", "packed-npm-runtime", "rust-build-lockfile", "repository-development-and-site", "editor-development", "collision-fixture", "mcp-client-fixture"]:
        require(security.get("scopes", {}).get(scope, {}).get("passed") is True, f"Security scope did not pass: {scope}")
    identity = security["artifact"]
    require(identity["packageReportSha256"] == file_record(package_report)["sha256"], "Security receipt names another package report")
    require(identity["tarballSha256"] == package["sha256"], "Package/security tarball hashes differ")
    require(identity["version"] == package["version"] and identity["buildRevision"] == package["runtime"]["revision"], "Package/security identities differ")
    require(identity["wasmSha256"] == package["runtime"]["reactiveWasmSha256"], "Package/security WASM hashes differ")
    require(package["runtime"].get("clean") is True and package["runtime"].get("compiled") is True, "A clean compiled candidate is required")
    require(package.get("checks", {}).get("browser") and package["checks"].get("library") is True, "Complete installed-package/browser checks are required")
    require(Path(package["tarball"]).name == package["tarball"], "Tarball must be beside report")
    tarball = package_report.parent / package["tarball"]
    require(file_record(tarball) == {"bytes": package["size"], "sha256": package["sha256"]}, "Tested tarball bytes changed")
    executable = Path(args.codex).resolve()
    require(executable.is_file() and executable.suffix.lower() not in {".cmd", ".bat", ".ps1"}, "--codex must name the real executable, not a shell shim")
    node = Path(shutil.which("node") or "").resolve()
    require(node.is_file(), "Node executable not found")
    npm = npm_command(node)
    study_before = study_manifest()
    run = ROOT / "test-results/agent-adoption" / (now().replace(":", "-") + f"-{os.getpid()}")
    run.mkdir(parents=True, exist_ok=False)
    inputs, preparation = run / "inputs", run / "preparation"
    inputs.mkdir()
    preparation.mkdir()
    copied_tarball = inputs / tarball.name
    shutil.copyfile(tarball, copied_tarball)
    shutil.copyfile(package_report, inputs / "package-report.json")
    shutil.copyfile(security_report, inputs / "security-report.json")
    env, excluded = child_environment()
    executable_receipts = {}
    for name, command in [("codex-version", [str(executable), "--version"]), ("node-version", [str(node), "--version"]), ("npm-version", npm + ["--version"])]:
        result = bounded(command, ROOT, preparation, name, 30, 1024 * 1024, env=env)
        require(command_passed(result), f"{name} failed; retained {run}")
        executable_receipts[name] = (preparation / f"{name}.stdout").read_text(encoding="utf-8").strip()
    trials = []
    common_packet = None
    common_package = None
    for trial_id in TRIAL_IDS:
        trial = run / trial_id
        trial.mkdir()
        shutil.copyfile(STUDY / "TASK.md", trial / "TASK.md")
        json_write(trial / "package.json", {"name": "caveat-adoption-trial", "version": "1.0.0", "private": True}, exclusive=True)
        install = bounded(npm + ["install", str(copied_tarball), "--offline", "--ignore-scripts", "--no-audit", "--no-fund"],
                          trial, preparation, f"install-{trial_id}", 120, 8 * 1024 * 1024, env=env)
        require(command_passed(install), f"Offline installation failed for {trial_id}; retained {run}")
        installed = trial / "node_modules/caveat-lang"
        require(json_read(installed / "package.json")["version"] == package["version"], "Installed package version differs")
        require(file_record(installed / "runtime/caveat_runtime_bg.wasm")["sha256"] == identity["wasmSha256"], "Installed WASM differs")
        packet, package_files = tree_manifest(trial, True), tree_manifest(installed)
        if common_packet is None:
            common_packet, common_package = packet, package_files
        require(packet == common_packet and package_files == common_package, "Trial packets are not identical")
        trials.append({"id": trial_id, "directory": str(trial), "command": codex_command(executable, trial)})
    # The scorer's current tests import checkout helpers. Verify those exact
    # runtime/helper inputs match the candidate before recording their result.
    candidate = Path(trials[0]["directory"]) / "node_modules/caveat-lang"
    scorer_inputs = {}
    for relative in ["lib/node.mjs", "lib/session.mjs", "lib/scenarios.mjs", "lib/check.mjs", "lib/explain.mjs", "examples/agent-evidence/assessment.cav"]:
        local = ROOT / "kit" / relative
        require(file_record(local) == file_record(candidate / relative), f"Scorer checkout input differs from candidate: {relative}")
        scorer_inputs[str(local)] = file_record(local)
    for name in ["caveat_runtime_bg.wasm", "caveat_runtime.js"]:
        local = ROOT / "dist/pkg-reactive" / name
        require(file_record(local) == file_record(candidate / "runtime" / name), f"Scorer runtime differs from candidate: {name}")
        scorer_inputs[str(local)] = file_record(local)
    controls = bounded([str(node), "--test", str(STUDY / "checks.test.mjs")], ROOT, preparation,
                       "scorer-controls", EVALUATOR_SECONDS, EVALUATOR_BYTES, env=env)
    require(command_passed(controls), f"Scorer validation failed; retained {run}")
    preflight = None
    observed_preflight = {}
    if args.preflight:
        source = Path(args.preflight).resolve()
        shutil.copyfile(source, inputs / "preflight.txt")
        observed_preflight = preflight_header(source.read_bytes())
        preflight = {"originalPath": str(source), "file": "inputs/preflight.txt", "observedHeader": observed_preflight, **file_record(source)}
    require(study_manifest() == study_before, "Study changed during preparation; freeze and prepare again")
    require(file_record(copied_tarball) == file_record(tarball), "Tarball changed during preparation")
    registration = {"schema": SCHEMA, "preparedAt": now(), "runDirectory": str(run), "status": "registered-not-run",
                    "candidate": {"name": package["name"], "version": package["version"], "tarball": f"inputs/{tarball.name}",
                                  "bytes": package["size"], "sha256": package["sha256"], "runtime": package["runtime"]},
                    "inputs": tree_manifest(inputs), "studyFiles": study_before, "scorerInputs": scorer_inputs,
                    "scorerValidation": {"receipt": "preparation/scorer-controls.receipt.json", **file_record(preparation / "scorer-controls.receipt.json")},
                    "tools": {"codex": {"path": str(executable), **file_record(executable)}, "node": {"path": str(node), **file_record(node)}, "versions": executable_receipts},
                    "configuration": {"modelOverride": None, "reasoningOverride": None, "observedPreflightModel": observed_preflight.get("model"),
                                      "observedPreflightReasoning": observed_preflight.get("reasoning effort"), "perTrialModelReported": None, "perTrialReasoningReported": None,
                                      "preflightEvidenceScope": "Equivalent non-JSON preflight only; JSON trials expose no configuration header.",
                                      "preflight": preflight, "preflightNotes": args.preflight_notes,
                                      "environmentExcludedNames": excluded, "environmentValuesRecorded": False,
                                      "boundary": "Shared host filesystem and caches; procedural packet boundary, not OS isolation. Project docs disabled; web search disabled; no tool-driven network required.",
                                      "approval": "--approve-for-me with windows.sandbox=elevated; no conflicting --sandbox flag"},
                    "limits": {"trialSeconds": TRIAL_SECONDS, "trialOutputBytes": TRIAL_BYTES,
                               "evaluatorSeconds": EVALUATOR_SECONDS, "evaluatorOutputBytes": EVALUATOR_BYTES},
                    "packetFiles": common_packet, "installedPackageFiles": common_package, "trials": trials,
                    "preparationFiles": tree_manifest(preparation),
                    "policy": "No intervention or evaluator feedback. No overwrite/restart. Retain infrastructure failures, timeouts, incomplete submissions and raw output."}
    json_write(run / "registration.json", registration, exclusive=True)
    (run / "registration.sha256").write_text(digest((run / "registration.json").read_bytes()) + "  registration.json\n", encoding="ascii")
    status("prepared", registration=str(run / "registration.json"), trialsStarted=0)


def validate_registration(file):
    file = Path(file).resolve()
    expected = (file.parent / "registration.sha256").read_text(encoding="ascii").split()[0]
    require(file_record(file)["sha256"] == expected, "Registration changed")
    registration = json_read(file)
    require(registration.get("schema") == SCHEMA and registration.get("status") == "registered-not-run", "Unsupported registration")
    run = Path(registration["runDirectory"]).resolve()
    require(file.parent == run, "Run the original registration in its prepared directory")
    require(study_manifest() == registration["studyFiles"], "Registered study files changed; prepare a new registration")
    require(tree_manifest(run / "inputs") == registration["inputs"], "Registered evidence inputs changed")
    require(tree_manifest(run / "preparation") == registration["preparationFiles"], "Preparation receipts changed")
    require(registration["limits"] == {"trialSeconds": TRIAL_SECONDS, "trialOutputBytes": TRIAL_BYTES, "evaluatorSeconds": EVALUATOR_SECONDS, "evaluatorOutputBytes": EVALUATOR_BYTES}, "Runner limits differ from registration")
    require([trial["id"] for trial in registration["trials"]] == list(TRIAL_IDS), "Unexpected trial set")
    for tool in ["node", "codex"]:
        record = registration["tools"][tool]
        require(file_record(record["path"]) == {key: record[key] for key in ["bytes", "sha256"]}, f"Registered {tool} executable changed")
    for file_name, expected_file in registration["scorerInputs"].items():
        require(file_record(file_name) == expected_file, f"Registered scorer input changed: {file_name}")
    control = registration["scorerValidation"]
    require(file_record(run / control["receipt"]) == {key: control[key] for key in ["bytes", "sha256"]}, "Scorer validation receipt changed")
    for trial in registration["trials"]:
        directory = Path(trial["directory"])
        require(directory.resolve() == run / trial["id"], "Unexpected trial directory")
        require(trial["command"] == codex_command(registration["tools"]["codex"]["path"], directory), "Trial command changed")
        require(tree_manifest(directory, True) == registration["packetFiles"], f"Trial packet changed: {trial['id']}")
        require(tree_manifest(directory / "node_modules/caveat-lang") == registration["installedPackageFiles"], f"Installed package changed: {trial['id']}")
    return registration, run


def run_trials(args):
    registration, run = validate_registration(args.registration)
    env, excluded = child_environment()
    require(excluded == registration["configuration"]["environmentExcludedNames"], "Context environment exclusions changed since preparation")
    json_write(run / "run-started.json", {"startedAt": now(), "registrationSha256": file_record(args.registration)["sha256"]}, exclusive=True)
    cancel = threading.Event()
    results = {}

    def trial_job(trial):
        directory = Path(trial["directory"])
        receipts = run / "external-receipts" / trial["id"]
        result = bounded(trial["command"], directory, receipts, "codex", TRIAL_SECONDS, TRIAL_BYTES,
                         stdin=(directory / "TASK.md").read_bytes(), env=env, cancel=cancel)
        json_write(receipts / "submission-manifest.json", tree_manifest(directory, True), exclusive=True)
        shutil.copytree(directory, receipts / "submission", symlinks=True, ignore=shutil.ignore_patterns("node_modules", "__pycache__"))
        return {"id": trial["id"], "trial": result, "receiptDirectory": str(receipts), "perTrialModelReported": None, "perTrialReasoningReported": None}

    status("trials-starting", registration=str(Path(args.registration).resolve()), trials=list(TRIAL_IDS))
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
        futures = {pool.submit(trial_job, trial): trial["id"] for trial in registration["trials"]}
        try:
            for future in concurrent.futures.as_completed(futures):
                trial_id = futures[future]
                try:
                    results[trial_id] = future.result()
                except Exception as error:
                    results[trial_id] = {"id": trial_id, "error": f"{type(error).__name__}: {error}"}
        except KeyboardInterrupt:
            cancel.set()
            status("runner-interrupted", action="terminating only owned trial processes")
            for future, trial_id in futures.items():
                try:
                    results[trial_id] = future.result()
                except Exception as error:
                    results[trial_id] = {"id": trial_id, "error": f"{type(error).__name__}: {error}"}
    # No evaluator feedback exists until every trial has stopped.
    for trial in registration["trials"]:
        trial_id, directory = trial["id"], Path(trial["directory"])
        result = results[trial_id]
        receipts = run / "external-receipts" / trial_id
        try:
            require(study_manifest() == registration["studyFiles"], "Scorer/study changed during trials")
            require(tree_manifest(directory / "node_modules/caveat-lang") == registration["installedPackageFiles"], "Trial modified its installed candidate; scoring cannot use those bytes")
            evaluator = bounded([registration["tools"]["node"]["path"], str(STUDY / "verify.mjs"), "--submission", str(directory),
                                 "--package", str(directory / "node_modules/caveat-lang")], directory, receipts,
                                "evaluator", EVALUATOR_SECONDS, EVALUATOR_BYTES, env=env)
            result["evaluator"] = evaluator
            if evaluator["stopReason"] is None and "error" not in evaluator:
                scored = json_read(receipts / "evaluator.stdout")
                require(scored.get("schema") == "caveat-agent-adoption-study/1", "Unexpected evaluator report")
                result["automated"] = scored["automated"]
            after = tree_manifest(directory, True)
            json_write(receipts / "post-evaluation-manifest.json", after, exclusive=True)
            result["submissionUnchangedByEvaluation"] = after == json_read(receipts / "submission-manifest.json")
            result["packageUnchangedByEvaluation"] = tree_manifest(directory / "node_modules/caveat-lang") == registration["installedPackageFiles"]
        except Exception as error:
            result["evaluationError"] = f"{type(error).__name__}: {error}"
        json_write(receipts / "result.json", result, exclusive=True)
        status("trial-recorded", trial=trial_id, evaluationError=result.get("evaluationError"), automated=result.get("automated"))
    summary = {"schema": "caveat-adoption-run/1", "finishedAt": now(), "registrationSha256": file_record(args.registration)["sha256"],
               "interrupted": cancel.is_set(), "results": [results[key] for key in TRIAL_IDS], "manualScores": None,
               "limitations": ["No manual rubric scores awarded by this runner.", "Fresh contexts share the host filesystem/caches; no filesystem secrecy or model-population claim."]}
    json_write(run / "run-summary.json", summary, exclusive=True)
    status("run-finished", summary=str(run / "run-summary.json"), trials=list(TRIAL_IDS))
    return 0 if all(command_passed(result.get("trial", {"exitCode": None, "stopReason": None})) and
                    result.get("automated", {}).get("finalObjectivesPass") is True and result.get("submissionUnchangedByEvaluation") is True and result.get("packageUnchangedByEvaluation") is True
                    for result in results.values()) else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_subparsers(dest="mode", required=True)
    prepare_parser = modes.add_parser("prepare", help="Install identical packets and register them; never starts agents")
    for argument in ["package-report", "security-report", "codex"]:
        prepare_parser.add_argument(f"--{argument}", required=True)
    prepare_parser.add_argument("--preflight", help="Existing observed CLI preflight header or receipt, copied and hashed")
    prepare_parser.add_argument("--preflight-notes", default="Parsed settings describe the supplied equivalent non-JSON preflight only; JSON trials expose no config header; no explicit model/effort override.")
    run_parser = modes.add_parser("run", help="Run exactly one previously reviewed registration; refuses restart")
    run_parser.add_argument("--registration", required=True)
    args = parser.parse_args()
    try:
        if args.mode == "prepare":
            prepare(args)
            return 0
        return run_trials(args)
    except Exception as error:
        status("failed", error=f"{type(error).__name__}: {error}")
        return 2


if __name__ == "__main__":
    sys.exit(main())
