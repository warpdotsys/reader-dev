"""Independently validate bounded offline soak reports, including every round.

This reads only small observations. It does not rerun a request, load an image,
read storage, or turn a finite generated-data sample into a production promise.
"""

import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re


SPEC = importlib.util.spec_from_file_location(
    "soak_report_cgroup", Path(__file__).with_name("report-browser-cgroup.py"))
CGROUP = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CGROUP)
MAX_JSON_BYTES = 128 * 1024
MAX_TRACE_BYTES = 1024 * 1024
MAX_ROUNDS = 1000


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def integer(value, minimum=0, maximum=None):
    return (type(value) is int and value >= minimum and
            (maximum is None or value <= maximum))


def duration(value):
    return (type(value) in (int, float) and math.isfinite(value) and value >= 0)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "DuplicateObservationKey")
        result[key] = value
    return result


def json_object(raw):
    def invalid_constant(_value):
        raise ValueError("NonFiniteObservation")
    value = json.loads(raw.decode("utf-8"), object_pairs_hook=unique_object,
                       parse_constant=invalid_constant)
    require(type(value) is dict, "ObservationMustBeObject")
    return value


def read_observation(directory, name, limit, digests):
    path = directory / name
    require(not path.is_symlink() and path.is_file(), "MissingRegularObservation")
    with path.open("rb") as stream:
        raw = stream.read(limit + 1)
    require(len(raw) <= limit, "ObservationByteLimitExceeded")
    digests[name] = hashlib.sha256(raw).hexdigest()
    return raw


def quiet(value):
    require(type(value) is dict, "MissingQuiescentObservation")
    require(value.get("browserProcesses") == {"worker": 0, "browser": 0, "driver": 0} and
            all(type(count) is int for count in value["browserProcesses"].values()),
            "SurvivingBrowserProcesses")
    require(integer(value.get("memoryPeakBytes"), 1, 2 * 1024 ** 3) and
            integer(value.get("memoryCurrentBytes"), 0, value["memoryPeakBytes"]) and
            integer(value.get("pidsPeak"), 1, 256) and
            integer(value.get("pidsCurrent"), 1, value["pidsPeak"]),
            "InvalidQuiescentResources")


def verify(directory, architecture, revision, source_revision, native_run,
           expected_jar, expected_image, seconds, require_container_state=False, require_removal=False):
    require(architecture in ("amd64", "arm64") and
            all(type(value) is str and re.fullmatch(r"[0-9a-f]{40}", value)
                for value in (revision, source_revision)) and
            integer(native_run, 1) and integer(seconds) and seconds in (600, 1800, 3600) and
            type(expected_jar) is str and re.fullmatch(r"[0-9a-f]{64}", expected_jar) and
            type(expected_image) is str and re.fullmatch(r"sha256:[0-9a-f]{64}", expected_image),
            "InvalidExpectedSoakIdentity")
    digests = {}
    def document(name):
        return json_object(read_observation(directory, name, MAX_JSON_BYTES, digests))
    source = document("SOURCE_RUN.json")
    require(type(source.get("id")) is int and source["id"] == native_run and
            source.get("conclusion") == "success" and source.get("head_sha") == source_revision and
            source.get("path") == ".github/workflows/release-native.yml" and
            source.get("name") == "Native release artifact rehearsal" and
            source.get("html_url") == f"https://github.com/warpdotsys/reader-dev/actions/runs/{native_run}",
            "SourceNativeRunMismatch")
    for filename, phase in (("PRELOAD_IDENTITY.json", "check"),
                            ("LOADED_IDENTITY.json", "loaded")):
        identity = document(filename)
        require(identity.get("phase") == phase and identity.get("architecture") == architecture and
                identity.get("revision") == revision and identity.get("jarSha256") == expected_jar and
                identity.get("imageId") == expected_image, "ImageOrJarIdentityMismatch")
    running = document("RUNNING_JAR_IDENTITY.json")
    require(running.get("revision") == revision and running.get("jarSha256") == expected_jar and
            running.get("network") == "none", "RunningJarOrNetworkMismatch")
    report = document("SOAK_REPORT.json")
    require(report.get("passed") is True and report.get("expectedImageRevision") == revision and
            type(report.get("requestedSeconds")) is int and report["requestedSeconds"] == seconds and
            type(report.get("accounts")) is int and report["accounts"] == 4 and
            integer(report.get("rounds"), 20, MAX_ROUNDS) and
            duration(report.get("continuousSeconds")) and report["continuousSeconds"] >= seconds and
            duration(report.get("totalSeconds")) and report["totalSeconds"] >= report["continuousSeconds"] and
            duration(report.get("coldRequestSeconds")) and
            "failedPhase" not in report and "failureType" not in report,
            "SoakDidNotComplete")
    release = report.get("releaseIdentity")
    require(type(release) is dict and release.get("buildRevision") == revision and
            type(release.get("version")) is str and len(release["version"]) <= 64 and
            re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?", release["version"]),
            "ReleaseIdentityMismatch")
    rounds = report["rounds"]
    require(type(report.get("successfulSearches")) is int and
            report["successfulSearches"] == 4 * rounds + 8 and
            type(report.get("targetCookieOrShapeErrors")) is int and report["targetCookieOrShapeErrors"] == 0 and
            report.get("targetMethods") == {"GET": 4 * ((rounds + 1) // 2) + 11, "POST": 4 * (rounds // 2)} and
            all(type(count) is int for count in report["targetMethods"].values()),
            "TargetOrSuccessfulRequestCountMismatch")
    faults = report.get("faults")
    require(type(faults) is list and len(faults) == 3, "MissingFaultRecovery")
    observations = [report.get("coldQuiescent"), report.get("warmQuiescent")]
    for fault, kind in zip(faults, ("navigationTimeout", "scriptWatchdog", "workerExit")):
        require(type(fault) is dict and fault.get("kind") == kind and
                fault.get("failedReturnDataVerified") is True and fault.get("healthyRecoveryVerified") is True and
                fault.get("ownedWorkerKilled") is (kind == "workerExit") and duration(fault.get("seconds")),
                "FaultRecoveryMismatch")
        observations.append(fault.get("afterRecovery"))
    raw_trace = read_observation(directory, "SOAK_TRACE.jsonl", MAX_TRACE_BYTES, digests)
    lines = raw_trace.splitlines()
    require(len(lines) == rounds, "TraceRoundCountMismatch")
    trace = []
    previous_elapsed = 0
    for index, line in enumerate(lines, 1):
        sample = json_object(line)
        require(type(sample.get("round")) is int and sample["round"] == index and
                sample.get("method") == ("GET" if index % 2 else "POST") and
                duration(sample.get("elapsedSeconds")) and sample["elapsedSeconds"] > previous_elapsed and
                duration(sample.get("maxRequestSeconds")) and
                sample["maxRequestSeconds"] <= sample["elapsedSeconds"] - previous_elapsed + 0.01,
                "InvalidContinuousRound")
        previous_elapsed = sample["elapsedSeconds"]
        trace.append(sample)
    # These are two actual monotonic-clock reads separated by flushing/closing
    # the last trace line. Keep both values; neither rounding nor a later idle
    # wait can substitute for the required duration of observed request rounds.
    require(trace[-1]["elapsedSeconds"] >= seconds and
            0 <= report["continuousSeconds"] - trace[-1]["elapsedSeconds"] <= 0.01 and
            report.get("samples") == trace[-5:], "SummaryTraceMismatch")
    observations.extend(trace)
    observations.append(report.get("finalQuiescent"))
    previous_memory_peak = previous_pids_peak = 0
    for observation in observations:
        quiet(observation)
        require(observation["memoryPeakBytes"] >= previous_memory_peak and
                observation["pidsPeak"] >= previous_pids_peak, "CumulativeResourcePeakRegressed")
        previous_memory_peak = observation["memoryPeakBytes"]
        previous_pids_peak = observation["pidsPeak"]
    resources = report.get("resources")
    require(type(resources) is dict and resources.get("architecture") ==
            ("x86_64" if architecture == "amd64" else "aarch64"), "ResourceArchitectureMismatch")
    require(type(resources.get("memoryMaxBytes")) is int and type(resources.get("pidsMax")) is int and
            integer(resources.get("cpuPeriod"), 1) and type(resources.get("cpuQuota")) is str and
            re.fullmatch(r"[0-9]{1,20}", resources["cpuQuota"]), "InvalidResourceBudgetTypes")
    try:
        CGROUP.verify_report(resources, require_no_swap=True)
    except (SystemExit, KeyError, TypeError, ValueError):
        raise ValueError("OriginalResourceBudgetRejected") from None
    quiet(dict(resources, browserProcesses={"worker": 0, "browser": 0, "driver": 0}))
    require(resources["memoryPeakBytes"] >= previous_memory_peak and
            resources["pidsPeak"] >= previous_pids_peak, "FinalResourcePeakRegressed")
    # The existing cgroup collector permits absent kernel-specific event fields.
    # A final soak acceptance requires these essential observations to be present.
    require(all(type(resources["memoryEvents"].get(key)) is int and resources["memoryEvents"][key] == 0
                for key in ("max", "oom", "oom_kill")) and
            type(resources.get("swapMaxBytes")) is int and resources["swapMaxBytes"] == 0 and
            type(resources.get("swapCurrentBytes")) is int and resources["swapCurrentBytes"] == 0 and
            type(resources["pidsEvents"].get("max")) is int and resources["pidsEvents"]["max"] == 0,
            "MissingMandatoryResourceCounters")
    observed_state = (directory / "CONTAINER_STATE.json").exists()
    require(not require_container_state or observed_state, "MissingPreRemovalContainerState")
    if observed_state:
        state = document("CONTAINER_STATE.json")
        require(state.get("Status") == "running" and state.get("Running") is True and
                all(state.get(key) is False for key in ("OOMKilled", "Dead", "Restarting", "Paused")) and
                type(state.get("ExitCode")) is int and state["ExitCode"] == 0 and state.get("Error") == "",
                "UnhealthyPreRemovalContainerState")
    observed_removal = (directory / "CONTAINER_REMOVAL.json").exists()
    require(not require_removal or observed_removal, "MissingPostRemovalInventory")
    if observed_removal:
        removal = document("CONTAINER_REMOVAL.json")
        container_id = running.get("containerId")
        require(type(container_id) is str and re.fullmatch(r"[0-9a-f]{64}", container_id) and
                removal.get("containerId") == container_id and removal.get("removalSucceeded") is True and
                removal.get("postRemovalInventoryObserved") is True and
                type(removal.get("ownedContainersRemaining")) is int and removal["ownedContainersRemaining"] == 0,
                "OwnedContainerRemovalUnproven")
    return {"accepted": True, "scope": "bounded offline generated-data soak only",
            "architecture": architecture, "nativeRun": native_run, "sourceRevision": source_revision,
            "imageRevision": revision, "jarSha256": expected_jar, "imageId": expected_image,
            "continuousSeconds": report["continuousSeconds"], "rounds": rounds,
            "lastRoundElapsedSeconds": trace[-1]["elapsedSeconds"],
            "successfulSearches": report["successfulSearches"], "memoryPeakBytes": resources["memoryPeakBytes"],
            "pidsPeak": resources["pidsPeak"],
            "maxObservedRequestSeconds": max(sample["maxRequestSeconds"] for sample in trace),
            "quiescentMemoryChangeBytes": report["finalQuiescent"]["memoryCurrentBytes"] - trace[0]["memoryCurrentBytes"],
            "preRemovalContainerStateObserved": observed_state,
            "independentPostRemovalInventoryObserved": observed_removal,
            "productionCapacityOrLeakAbsenceProven": False, "fileSha256": digests}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--architecture", required=True, choices=("amd64", "arm64"))
    parser.add_argument("--expected-revision", required=True)
    parser.add_argument("--expected-source-revision", required=True)
    parser.add_argument("--expected-native-run", required=True, type=int)
    parser.add_argument("--expected-jar", required=True)
    parser.add_argument("--expected-image", required=True)
    parser.add_argument("--seconds", required=True, type=int, choices=(600, 1800, 3600))
    parser.add_argument("--require-container-state", action="store_true")
    parser.add_argument("--require-removal", action="store_true")
    args = parser.parse_args()
    try:
        result = verify(args.directory, args.architecture, args.expected_revision,
                        args.expected_source_revision, args.expected_native_run, args.expected_jar,
                        args.expected_image, args.seconds, args.require_container_state, args.require_removal)
    except (ValueError, OSError, KeyError, TypeError, RecursionError):
        # Do not emit a raw server response, JSON field, path or parser traceback.
        print(json.dumps({"accepted": False, "reason": "OfflineSoakEvidenceRejected"}))
        raise SystemExit(1) from None
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
