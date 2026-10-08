#!/usr/bin/env python3
"""Consume a tested image for generated-only comparisons; never rebuild or publish it."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
ORIGINAL_SHA = "b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c"
ORIGINAL_IMAGE = "swr.cn-north-4.myhuaweicloud.com/ddn-k8s/docker.io/hectorqin/reader:3.2.14"
ARCHIVED = "hectorqin/remote-webview@sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a"
EXPECTED_JOBS = {"Build one shared Java Kotlin JAR", "Build and test native amd64 image",
    "Build and test native arm64 image", "Reload and run tested amd64 image",
    "Reload and run tested arm64 image", "Exercise both publisher imports without registry credentials"}


def command(*args, timeout=60, check=True):
    return subprocess.run(list(map(str, args)), cwd=ROOT, capture_output=True, text=True,
                          timeout=timeout, check=check)


def read(path):
    if not path.is_file() or path.is_symlink() or path.stat().st_size > 1048576:
        raise ValueError("Expected a small regular evidence file")
    return json.loads(path.read_text(encoding="utf-8"))


def write_new(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def validate_source(run, jobs, comparison, run_id, source, revision):
    if not re.fullmatch(r"[0-9a-f]{40}", source) or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("Exact source and tested snapshot required")
    if type(run_id) is not int or run_id <= 0 or run.get("id") != run_id or run.get("head_sha") != source:
        raise ValueError("Wrong native run identity")
    if run.get("status") != "completed" or run.get("conclusion") != "success" or \
            run.get("path") != ".github/workflows/release-native.yml" or \
            run.get("name") != "Native release artifact rehearsal" or \
            run.get("head_repository", {}).get("full_name") != "warpdotsys/reader-dev" or \
            run.get("event") not in ("pull_request", "push", "workflow_dispatch"):
        raise ValueError("Only the completed own native rehearsal may be consumed")
    rows = jobs.get("jobs", [])
    if jobs.get("total_count") != 6 or len(rows) != 6 or {row.get("name") for row in rows} != EXPECTED_JOBS or \
            any(type(row.get("id")) is not int or row["id"] <= 0 for row in rows) or \
            len({row["id"] for row in rows}) != 6 or \
            any(row.get("status") != "completed" or row.get("conclusion") != "success" for row in rows):
        raise ValueError("All six actual native jobs must have succeeded")
    if comparison.get("behind_by") != 0 or comparison.get("ahead_by") != (0 if source == revision else 1) or \
            comparison.get("files") != [] or comparison.get("base_commit", {}).get("sha") != source or \
            [row.get("sha") for row in comparison.get("commits", [])] != ([] if source == revision else [revision]):
        raise ValueError("Native source and tested snapshot must have identical file contents")


def require_hosted(environment):
    if sys.platform != "linux" or environment.get("GITHUB_ACTIONS") != "true" or \
            environment.get("RUNNER_ENVIRONMENT") != "github-hosted" or \
            environment.get("RUNNER_OS") != "Linux" or environment.get("RUNNER_ARCH") != "X64" or \
            environment.get("GITHUB_REPOSITORY") != "warpdotsys/reader-dev":
        raise ValueError("Only the own native AMD64 GitHub-hosted runner is allowed")
    for key in ("GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT"):
        if not re.fullmatch(r"[1-9][0-9]*", environment.get(key, "")):
            raise ValueError("Exact probe run ownership required")
    if Path(environment.get("GITHUB_WORKSPACE", "absent")).resolve() != ROOT:
        raise ValueError("Wrong checkout")


def observe_removal(provenance):
    slice_name = provenance.get("slice", "")
    if not re.fullmatch(r"readerwebview[0-9a-f]{12}\.slice", slice_name):
        raise ValueError("No verified owned test slice")
    token = slice_name.removesuffix(".slice")
    removed = provenance.get("removedOwnedTestContainers", [])
    if len(removed) != 2 or len(set(removed)) != 2 or any(not re.fullmatch(r"[0-9a-f]{64}", cid) for cid in removed):
        raise ValueError("Both exact test containers must have been removed")
    remaining = command("docker", "ps", "-aq", "--no-trunc", "--filter",
                        "label=com.medwarp.reader.generated-test=" + token).stdout.split()
    group = Path("/sys/fs/cgroup") / slice_name
    pids = set()
    if group.exists():
        for file in group.rglob("cgroup.procs"):
            for pid in file.read_text().split():
                if not pid.isdigit():
                    raise ValueError("Invalid owned cgroup process observation")
                pids.add(int(pid))
    if remaining or pids:
        raise RuntimeError("Owned test containers or descendants remain after cleanup")
    return {"postRemovalHostInventoryObserved": True, "ownedContainersRemaining": 0,
            "ownedCgroupProcessesRemaining": 0, "sliceAbsent": not group.exists(), "containerIds": removed}


def validate_metadata_outcome(provenance, comparison, jar, image):
    for key in ("probeCompleted", "budgetAccepted", "overallAccepted", "originalJarUnchanged",
                "restoredJarUnchanged", "historicalRendererStoppedBeforeCamoufox"):
        if provenance.get(key) is not True:
            raise ValueError("Incomplete actual probe: " + key)
    if provenance.get("originalJarSha256") != ORIGINAL_SHA or provenance.get("restoredJarSha256") != jar or \
            provenance.get("runtimeImageId") != image or provenance.get("archivedRenderer") != ARCHIVED:
        raise ValueError("Comparison input identity differs")
    budget = provenance.get("aggregateBudget", {})
    if budget.get("cpuQuota") != 200000 or budget.get("cpuPeriod") != 100000 or \
            budget.get("memoryMaxBytes") != 2147483648 or budget.get("swapMaxBytes") != 0 or budget.get("pidsMax") != 512 or \
            any(budget.get("memoryEvents", {}).get(key) != 0 for key in ("max", "oom", "oom_kill")) or \
            budget.get("pidsEvents", {}).get("max") != 0:
        raise ValueError("Aggregate no-swap resource gate is not proven")
    if comparison.get("originalExecuted") is not True or comparison.get("comparisonMode") != "same-run-original-remote-camoufox" or \
            comparison.get("originalJarSha256") != ORIGINAL_SHA or comparison.get("restoredJarSha256") != jar or \
            comparison.get("camoufoxJarSha256") != jar or comparison.get("fullGeneratedReaderJsonRecorded") is not True or \
            comparison.get("metadataProbe", {}).get("actualGetBookInfo") is not True:
        raise ValueError("Actual three-way JSON and metadata observations are required")
    for side in ("original", "restored", "camoufox"):
        if not isinstance(comparison.get(side), dict) or len(comparison[side].get("searches", [])) != 5 or \
                comparison[side].get("metadata", {}).get("bookInfoApiCalls") != 1:
            raise ValueError("All three actual generated journeys must finish")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-run", type=int, required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--mode", choices=("metadata", "utf8-characterization"), required=True)
    parser.add_argument("--validate-source-only", action="store_true")
    args = parser.parse_args()
    require_hosted(os.environ)  # Before Docker, sudo, any inputs or private filesystem access.
    report = Path(os.environ["RUNNER_TEMP"]) / "reader-three-way-report"
    validate_source(read(report / "SOURCE_RUN.json"), read(report / "SOURCE_JOBS.json"),
                    read(report / "SOURCE_COMPARISON.json"), args.native_run, args.source, args.revision)
    if args.validate_source_only:
        print(json.dumps({"sourceNativeRunId": args.native_run, "sourceRevision": args.source,
                          "testedRuntimeRevision": args.revision, "sourceValidatedBeforeDownload": True}))
        return
    if shutil.disk_usage(report).free < 10 * 1024 ** 3:
        raise RuntimeError("Insufficient hosted disk; do not prune unowned images")
    if command("docker", "info", "--format", "{{.CgroupDriver}}").stdout.strip() != "systemd":
        raise RuntimeError("Existing aggregate harness requires systemd cgroups; do not reconfigure Docker")
    meta = read(ROOT / "imported/metadata.json")
    version = meta.get("version", "")
    preloaded = json.loads(command("node", "scripts/release-native-artifacts.mjs", "check",
        "amd64", version, args.revision, "dist", "imported", timeout=300).stdout)
    write_new(report / "PRELOAD_IDENTITY.json", preloaded)
    command("docker", "load", "--input", "imported/reader-image.tar.gz", timeout=420)
    inspected = command("docker", "image", "inspect", "reader-pro:camoufox-smoke-amd64").stdout
    with (ROOT / "imported/image-inspect.json").open("x", encoding="utf-8") as stream:
        stream.write(inspected)
    loaded = json.loads(command("node", "scripts/release-native-artifacts.mjs", "loaded",
        "amd64", version, args.revision, "dist", "imported", timeout=300).stdout)
    write_new(report / "LOADED_IDENTITY.json", loaded)
    image = meta["imageId"]
    if loaded.get("imageId") != image or loaded.get("jarSha256") != meta["jarSha256"]:
        raise ValueError("Loaded identity does not match this native artifact")
    command("docker", "pull", "--platform", "linux/amd64", ORIGINAL_IMAGE, timeout=420)
    label = os.environ["GITHUB_RUN_ID"] + "-" + os.environ["GITHUB_RUN_ATTEMPT"]
    cid = command("docker", "create", "--platform", "linux/amd64", "--label",
        "com.medwarp.reader.three-way-original=" + label, "--entrypoint", "/bin/true", ORIGINAL_IMAGE).stdout.strip()
    if not re.fullmatch(r"[0-9a-f]{64}", cid):
        raise ValueError("Invalid exact extraction-container identity")
    original = report / "original.jar"
    try:
        command("docker", "cp", cid + ":/app/bin/reader.jar", original, timeout=120)
        if not original.is_file() or original.is_symlink() or not 0 < original.stat().st_size < 536870912 or digest(original) != ORIGINAL_SHA:
            raise ValueError("Public archive JAR differs from the untouched local baseline")
        original.chmod(0o444)
    finally:
        state = json.loads(command("docker", "inspect", cid).stdout)[0]
        if state["Config"]["Labels"].get("com.medwarp.reader.three-way-original") != label or state["State"]["Running"]:
            raise RuntimeError("Refusing cleanup of an unowned or started extraction container")
        command("docker", "rm", cid)
    command("docker", "pull", "--platform", "linux/amd64", ARCHIVED, timeout=420)
    jars = list((ROOT / "dist").glob("reader-*.jar"))
    if len(jars) != 1 or jars[0].is_symlink() or digest(jars[0]) != meta["jarSha256"]:
        raise ValueError("Only the exact shared candidate JAR may be compared")
    output = Path("/var/tmp") / ("reader-three-way-" + label + "-" + args.mode)
    flags = ["--exercise-metadata", "--metadata-clock-contract"] if args.mode == "metadata" else [
        "--exercise-encoding", "--characterize-historical-utf8"]
    # Root-owned GNU timeout sends TERM to its own process group first. The
    # existing harness handles TERM/HUP and cleans its exact owned containers.
    result = command("sudo", "-n", "timeout", "--signal=TERM", "--kill-after=15s", "480s",
        "python3", "scripts/run-three-way-webview-in-docker.py",
        "--original", original, "--restored", jars[0], "--runtime-image", image,
        "--output", output, *flags, timeout=520, check=False)
    # Copy only small generated JSON, never the JAR, archive, full logs or data directories.
    for origin, name in ((output / "provenance.json", "PROVENANCE.json"),
                         (output / "results/three-way.json", "THREE_WAY.json")):
        if origin.exists():
            write_new(report / name, read(origin))
    provenance = read(report / "PROVENANCE.json")
    removal = observe_removal(provenance)
    write_new(report / "POST_REMOVAL.json", removal)
    accepted = args.mode == "metadata" and result.returncode == 0 and provenance.get("overallAccepted") is True
    if accepted:
        validate_metadata_outcome(provenance, read(report / "THREE_WAY.json"), meta["jarSha256"], image)
    summary = {"generatedOnly": True, "sourceNativeRunId": args.native_run, "sourceRevision": args.source,
        "testedRuntimeRevision": args.revision, "probeCheckoutRevision": os.environ["GITHUB_SHA"],
        "mode": args.mode, "probeExitCode": result.returncode, "overallAccepted": accepted,
        "originalJarSha256": digest(original), "candidateJarSha256": digest(jars[0]), "imageId": image,
        "originalJarUnchanged": provenance.get("originalJarUnchanged"),
        "candidateJarUnchanged": provenance.get("restoredJarUnchanged"), "postRemoval": removal,
        "realCredentialsImported": False, "privateBookBodiesRead": 0, "productionChanged": False,
        "originalProductionRendererVersionProven": False}
    write_new(report / "SUMMARY.json", summary)
    print(json.dumps(summary))
    if not accepted:
        raise RuntimeError("Preserved a failed/diagnostic comparison; strict compatibility is not accepted")


if __name__ == "__main__":
    main()
