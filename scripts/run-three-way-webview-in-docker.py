#!/usr/bin/env python3
"""Generated-only three-way probe on Linux, with one aggregate resource budget.

Inputs stay on this host. No production mount, published port, host PID namespace
or privileged container is used. Requires the existing pinned full Reader image
and archived renderer to have been downloaded before this offline run.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time

ARCHIVED = "hectorqin/remote-webview@sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a"
ORIGINAL_SHA = "b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c"
SCRIPTS = Path(__file__).resolve().parent
CPU_STAT_FIELDS = ("usage_usec", "user_usec", "system_usec", "nr_periods", "nr_throttled", "throttled_usec")


def command(*args, timeout=30, check=True):
    return subprocess.run(list(map(str, args)), capture_output=True, text=True,
                          timeout=timeout, check=check)


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def write_new(path, value):
    with path.open("x", encoding="utf-8") as output:
        json.dump(value, output, ensure_ascii=False, indent=2)
        output.write("\n")


def budget_snapshot(group):
    def pairs(name):
        return {key: int(value) for key, value in
                (line.split() for line in (group / name).read_text().splitlines())}
    quota, period = (group / "cpu.max").read_text().split()
    result = {"scope": "both test containers and all their probe/Java/browser descendants",
              "cpuQuota": int(quota), "cpuPeriod": int(period),
              "memoryMaxBytes": int((group / "memory.max").read_text()),
              "memoryPeakBytes": int((group / "memory.peak").read_text()),
              "memoryEvents": pairs("memory.events"),
              "swapMaxBytes": int((group / "memory.swap.max").read_text()),
              "pidsMax": int((group / "pids.max").read_text()), "pidsEvents": pairs("pids.events")}
    if result["cpuQuota"] != result["cpuPeriod"] * 2 or \
            result["memoryMaxBytes"] != 2147483648 or result["swapMaxBytes"] != 0 or \
            result["pidsMax"] != 512:
        raise RuntimeError("The aggregate resource budget was not actually applied")
    return result


def require_unpressured_budget(report):
    if any(report["memoryEvents"].get(key, 0) for key in ("max", "oom", "oom_kill")) or \
            report["pidsEvents"].get("max", 0):
        raise RuntimeError("The generated test triggered an aggregate resource limit")


def require_probe_acceptance(exit_code, characterize_historical_utf8=False):
    if type(exit_code) is not int or exit_code != 0:
        raise RuntimeError("Three-way probe failed; preserved generated report and diagnostics")
    if characterize_historical_utf8:
        # A diagnostic must never turn either a known defect or an unexpected
        # zero exit into overallAccepted=true in this strict acceptance harness.
        raise RuntimeError("Historical UTF-8 characterization is not strict acceptance")


def cpu_observation(group):
    """Optional actual counters, never a substitute for the mandatory budget."""
    stat = group / "cpu.stat"
    pressure = group / "cpu.pressure"
    counters = None
    if stat.exists():
        counters = {name: None for name in CPU_STAT_FIELDS}
        for line in stat.read_text().splitlines():
            key, value = line.split()
            if key in counters:
                if counters[key] is not None or not value.isdigit():
                    raise ValueError("Invalid CPU counter")
                counters[key] = int(value)
    observations = None
    if pressure.exists():
        observations = {}
        for line in pressure.read_text().splitlines():
            parts = line.split()
            if not parts or parts[0] not in ("some", "full") or parts[0] in observations:
                raise ValueError("Invalid CPU pressure kind")
            values = {}
            for entry in parts[1:]:
                key, value = entry.split("=")
                if key in values or key not in ("avg10", "avg60", "avg300", "total"):
                    raise ValueError("Invalid CPU pressure field")
                if key == "total":
                    if not value.isdigit():
                        raise ValueError("Invalid CPU pressure total")
                    values[key] = int(value)
                else:
                    value = float(value)
                    if not math.isfinite(value) or not 0 <= value <= 100:
                        raise ValueError("Invalid CPU pressure average")
                    values[key] = value
            if set(values) != {"avg10", "avg60", "avg300", "total"}:
                raise ValueError("Incomplete CPU pressure observation")
            observations[parts[0]] = values
    return {"cpuStat": counters, "cpuPressure": observations}


def resource_sample(group, phase, started):
    memory = {key: int(value) for key, value in
              (line.split() for line in (group / "memory.stat").read_text().splitlines())}
    events = {key: int(value) for key, value in
              (line.split() for line in (group / "memory.events").read_text().splitlines())}
    return {"phase": phase, "elapsedMs": round((time.monotonic() - started) * 1000),
            "memoryCurrentBytes": int((group / "memory.current").read_text()),
            "memoryAnonBytes": memory["anon"], "memoryFileBytes": memory["file"],
            "memoryShmemBytes": memory["shmem"], "memoryEvents": events,
            "pidsCurrent": int((group / "pids.current").read_text()), **cpu_observation(group)}


def owned_container(cid, token):
    state = json.loads(command("docker", "inspect", cid).stdout)[0]
    if state["Config"]["Labels"].get("com.medwarp.reader.generated-test") != token:
        raise RuntimeError("Refusing to act on a container without this run's ownership label")
    return state


def preserve_owned_renderer_diagnostics(cid, token, output):
    """Private bounded generated-only logs, captured before removing this owned container."""
    state = owned_container(cid, token)["State"]
    observed = {key: state[key] for key in ("Running", "Paused", "Restarting", "OOMKilled", "ExitCode", "Pid")}
    if any(type(observed[key]) is not bool for key in ("Running", "Paused", "Restarting", "OOMKilled")) or \
            any(type(observed[key]) is not int for key in ("ExitCode", "Pid")):
        raise ValueError("Invalid owned renderer state")
    logs = subprocess.run(["docker", "logs", "--tail=80", cid], capture_output=True, timeout=10, check=False)
    evidence = {"generatedOnly": True, "capturedBeforeOwnedCleanup": True, "state": observed,
                "logCaptureExitCode": logs.returncode, "logs": {}}
    for name, raw in (("stdout", logs.stdout), ("stderr", logs.stderr)):
        stored = raw[-32768:]
        path = output / ("archived-renderer-" + name + ".log")
        descriptor = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(stored)
        evidence["logs"][name] = {"path": path.name, "capturedBytes": len(raw), "storedBytes": len(stored),
            "truncated": len(stored) != len(raw), "storedSha256": hashlib.sha256(stored).hexdigest()}
    return evidence


def run_probe_with_handoff(probe, log_path, phases, handoff, sample, timeout=360):
    """Only permit Camoufox after the actual historical pair completed and stopped."""
    handed_off = False
    next_sample = 0
    with log_path.open("x", encoding="utf-8") as log:
        process = subprocess.Popen(probe, stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + timeout
            while process.poll() is None:
                if time.monotonic() >= deadline:
                    raise TimeoutError("Three-way probe exceeded its bounded runtime")
                ready = phases / "remote-complete"
                if not handed_off and (ready.exists() or ready.is_symlink()):
                    if ready.is_symlink() or not ready.is_file() or ready.stat().st_size != 0:
                        raise RuntimeError("Invalid completed historical pair marker")
                    handoff()
                    with (phases / "camoufox-permitted").open("xb"):
                        pass
                    handed_off = True
                now = time.monotonic()
                if now >= next_sample:
                    sample("camoufox" if handed_off else "historical-pair")
                    next_sample = now + 1
                time.sleep(0.1)
            if process.returncode == 0 and not handed_off:
                raise RuntimeError("Probe claimed success without the renderer handoff")
            return process.returncode
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--original", type=Path, required=True)
    parser.add_argument("--restored", type=Path, required=True)
    parser.add_argument("--runtime-image", required=True, help="Already verified sha256 image ID")
    parser.add_argument("--output", type=Path, required=True, help="New directory under /var/tmp")
    parser.add_argument("--exercise-encoding", action="store_true",
                        help="Also execute the sixth generated UTF-8 POST/script/response comparison")
    parser.add_argument("--exercise-metadata", action="store_true",
                        help="Add generated delayed-DOM getBookInfo parity after the unchanged five baseline cases")
    parser.add_argument("--metadata-clock-contract", action="store_true",
                        help="Explicit metadata default-clock freshness contract; not literal timestamp equality")
    parser.add_argument("--characterize-historical-utf8", action="store_true",
                        help="Diagnostic only: preserve all sides of the known UTF-8 defect, never overall acceptance")
    args = parser.parse_args()
    if args.characterize_historical_utf8 and not args.exercise_encoding:
        parser.error("Historical UTF-8 characterization requires --exercise-encoding")
    if args.exercise_metadata and (args.exercise_encoding or args.characterize_historical_utf8):
        parser.error("Metadata addition does not replace or relax the separate strict UTF-8 gate")
    if args.metadata_clock_contract and not args.exercise_metadata:
        parser.error("Metadata clock contract requires --exercise-metadata")
    if sys.platform != "linux" or os.geteuid() != 0:
        parser.error("A root-owned Linux Docker/systemd host is required")
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", args.runtime_image):
        parser.error("Runtime image must be an exact image ID, never a mutable tag")
    output = args.output.resolve()
    if not output.is_relative_to(Path("/var/tmp")) or output == Path("/var/tmp") or \
            args.output.is_symlink() or output.exists():
        parser.error("Choose a new output directory strictly below /var/tmp")
    original, restored = args.original.resolve(strict=True), args.restored.resolve(strict=True)
    if not original.is_file() or not restored.is_file() or digest(original) != ORIGINAL_SHA:
        parser.error("Read-only original JAR does not match the baseline")
    if command("docker", "info", "--format", "{{.CgroupDriver}}").stdout.strip() != "systemd":
        parser.error("Aggregate slice isolation requires the systemd cgroup driver")
    image = json.loads(command("docker", "image", "inspect", args.runtime_image).stdout)[0]
    if image["Id"] != args.runtime_image or image["Architecture"] != "amd64" or \
            image["Config"]["User"] != "10001:10001":
        parser.error("Expected the previously tested non-root amd64 runtime image")
    command("docker", "image", "inspect", ARCHIVED)
    token = "readerwebview" + os.urandom(6).hex()
    parent, anchor = token + ".slice", token + "-anchor.service"
    group = Path("/sys/fs/cgroup") / parent
    containers = []
    legacy = None
    slice_started = False
    output.mkdir(mode=0o755)
    results = output / "results"
    results.mkdir(mode=0o750)
    os.chown(results, 10001, 10001)
    provenance = {"generatedOnly": True, "originalJarSha256": digest(original),
                  "restoredJarSha256": digest(restored), "runtimeImageId": args.runtime_image,
                  "archivedRenderer": ARCHIVED, "realAuthenticatedSourceTested": False,
                  "originalProductionRendererVersionProven": False, "slice": parent,
                  "containerCgroups": [], "probeCompleted": False,
                  "budgetAccepted": False, "overallAccepted": False,
                  "executionLayout": "serial-renderers-shared-fixture",
                  "historicalRendererStoppedBeforeCamoufox": False, "resourceSamples": []}
    if args.exercise_encoding:
        provenance["encodingProbeRequested"] = True
    if args.exercise_metadata:
        provenance["metadataProbeRequested"] = True
        provenance["metadataComparisonScope"] = "exact-static-json-and-bounded-default-clocks" if args.metadata_clock_contract else "literal-full-json"
    if args.metadata_clock_contract:
        provenance["metadataClockContractRequested"] = True
    if args.characterize_historical_utf8:
        provenance["historicalUtf8CharacterizationRequested"] = True
    try:
        command("systemd-run", "--unit=" + anchor, "--slice=" + parent,
                "--property=Type=oneshot", "--property=RemainAfterExit=yes", "/bin/true")
        slice_started = True
        command("systemctl", "set-property", "--runtime", parent, "CPUQuota=200%",
                "MemoryMax=2147483648", "MemorySwapMax=0", "TasksMax=512")
        budget_snapshot(group)
        common = ["--cgroup-parent", parent, "--cpus=2", "--memory=2g", "--memory-swap=2g",
                  "--pids-limit=256", "--shm-size=512m", "--cap-drop=ALL",
                  "--security-opt=no-new-privileges:true", "--restart=no",
                  "--label", "com.medwarp.reader.generated-test=" + token]

        def start(name, arguments):
            cid = command("docker", "run", "-d", "--name", token + "-" + name,
                          *common, *arguments).stdout.strip()
            if not re.fullmatch(r"[0-9a-f]{64}", cid):
                raise RuntimeError("Unexpected test container identity")
            containers.append(cid)
            state = json.loads(command("docker", "inspect", cid).stdout)[0]
            path = Path("/proc") / str(state["State"]["Pid"]) / "cgroup"
            membership = path.read_text().strip()
            if not membership.startswith("0::/" + parent + "/") or state["HostConfig"]["PortBindings"]:
                raise RuntimeError("A test component escaped its aggregate slice or published ports")
            provenance["containerCgroups"].append(membership)
            return cid, state["State"]["Pid"]

        legacy, legacy_pid = start("legacy", ["--network=none", ARCHIVED])
        # This guard executes on the host with only its network namespace changed.
        # PID 1 remains the actual host PID 1, not the container's private PID 1.
        verify = ("import importlib.util,os,sys; s=importlib.util.spec_from_file_location('guard',sys.argv[1]);"
                  "m=importlib.util.module_from_spec(s);s.loader.exec_module(m);"
                  "m.require_private_loopback();print(os.stat('/proc/self/ns/net').st_ino)")
        guard_path = SCRIPTS / "run-webview-cookie-in-linux-netns.py"
        inode = command("nsenter", "--target", legacy_pid, "--net", "--", "python3", "-c",
                        verify, guard_path).stdout.strip()
        if not inode.isdigit():
            raise RuntimeError("The independent root namespace guard failed")
        provenance["privateNetworkNamespaceInode"] = int(inode)
        runtime, runtime_pid = start("runtime", [
            "--network=container:" + legacy, "--user=10001:10001", "--init",
            "--tmpfs", "/tmp:size=512m,mode=1777", "--entrypoint=/usr/bin/python3",
            "--mount", "type=bind,src=" + str(SCRIPTS) + ",dst=/probe,readonly",
            "--mount", "type=bind,src=" + str(original) + ",dst=/inputs/original.jar,readonly",
            "--mount", "type=bind,src=" + str(restored) + ",dst=/inputs/restored.jar,readonly",
            "--mount", "type=bind,src=" + str(results) + ",dst=/results",
            args.runtime_image, "-c", "import time;time.sleep(600)"])
        if os.stat("/proc/" + str(runtime_pid) + "/ns/net").st_ino != int(inode):
            raise RuntimeError("Probe container does not share the guarded private network")
        readiness = "import socket; socket.create_connection(('127.0.0.1',8050),timeout=2).close()"
        for attempt in range(30):
            if command("docker", "exec", runtime, "python3", "-c", readiness, check=False).returncode == 0:
                break
            time.sleep(1)
        else:
            raise RuntimeError("Archived renderer did not become ready in private loopback")
        probe = ["docker", "exec", "--user=10001:10001",
                 "--env=READER_ORIGINAL_JAR_NETWORK_ISOLATED=confirmed",
                 "--env=READER_PRIVATE_NETNS_INODE=" + inode, runtime,
                 "python3", "/probe/compare-webview-cookie.py",
                 "--java", "/opt/java/openjdk/bin/java", "--original", "/inputs/original.jar",
                 "--restored", "/inputs/restored.jar", "--report", "/results/three-way.json",
                 "--original-network-isolated", "--exercise-script", "--exercise-post",
                 "--archived-renderer-base", "http://127.0.0.1:8050",
                 "--camoufox-python", "/usr/bin/python3", "--phase-handoff-dir", "/results/phases"]
        if args.exercise_encoding:
            probe.append("--exercise-encoding")
        if args.exercise_metadata:
            probe.append("--exercise-metadata")
        if args.metadata_clock_contract:
            probe.append("--metadata-clock-contract")
        if args.characterize_historical_utf8:
            probe.append("--characterize-historical-utf8")
        started = time.monotonic()

        def sample(phase):
            provenance["resourceSamples"].append(resource_sample(group, phase, started))

        def handoff():
            sample("historical-pair-complete")
            # Never reset cumulative memory.peak/events or increase the shared budget.
            require_unpressured_budget(budget_snapshot(group))
            owned_container(legacy, token)
            command("docker", "stop", "--time=10", legacy, timeout=30)
            state = owned_container(legacy, token)
            if state["State"]["Running"]:
                raise RuntimeError("Owned historical renderer did not stop before Camoufox")
            # The runtime holds the same netns after the historical renderer exits.
            guarded = command("nsenter", "--target", runtime_pid, "--net", "--", "python3", "-c",
                              verify, guard_path).stdout.strip()
            if guarded != inode:
                raise RuntimeError("Private network changed during the renderer handoff")
            provenance["historicalRendererStoppedBeforeCamoufox"] = True
            sample("historical-renderer-stopped")

        sample("probe-start")
        exit_code = run_probe_with_handoff(probe, output / "probe.log", results / "phases",
                                          handoff, sample)
        sample("probe-finished")
        provenance["probeExitCode"] = exit_code
        provenance["probeCompleted"] = exit_code == 0
        provenance["aggregateBudget"] = budget_snapshot(group)
        require_probe_acceptance(exit_code, args.characterize_historical_utf8)
        events = provenance["aggregateBudget"]
        require_unpressured_budget(events)
        provenance["budgetAccepted"] = True
        provenance["overallAccepted"] = True
        print(json.dumps({"probeCompleted": True, "aggregateBudget": events}), flush=True)
    finally:
        if group.exists() and "aggregateBudget" not in provenance:
            try:
                provenance["aggregateBudget"] = budget_snapshot(group)
            except (OSError, ValueError, RuntimeError):
                provenance["aggregateBudgetUnavailable"] = True
        # Preserve only this run's generated engine logs. Diagnostic failure must
        # not skip exact ownership checks, mandatory cleanup or change acceptance.
        if legacy is not None:
            try:
                provenance["archivedRendererDiagnostics"] = preserve_owned_renderer_diagnostics(legacy, token, output)
            except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
                provenance["archivedRendererDiagnosticFailureType"] = type(error).__name__
        stopped = []
        for cid in reversed(containers):
            owned_container(cid, token)
            command("docker", "rm", "--force", cid)
            stopped.append(cid)
        provenance["removedOwnedTestContainers"] = stopped
        if slice_started:
            command("systemctl", "stop", anchor, parent)
        provenance["originalJarUnchanged"] = digest(original) == ORIGINAL_SHA
        provenance["restoredJarUnchanged"] = digest(restored) == provenance["restoredJarSha256"]
        write_new(output / "provenance.json", provenance)


if __name__ == "__main__":
    def interrupted(_signal, _frame):
        raise SystemExit("Interrupted; cleaning up owned test containers")
    signal.signal(signal.SIGTERM, interrupted)
    if hasattr(signal, "SIGHUP"):
        signal.signal(signal.SIGHUP, interrupted)
    main()
