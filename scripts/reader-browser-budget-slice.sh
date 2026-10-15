#!/usr/bin/env bash
# Generated offline soak on a fresh GitHub-hosted runner; CLI preflight is read-only.
# This is an opt-in parent policy, not an image/heap/font or production change.
budget_slice=''
budget_anchor=''
budget_cgroup=''
budget_started=false
budget_args=()

require_unconfigured_slice_state() {
  # systemd synthesizes .slice units even without a unit file. "loaded" alone
  # is not an existing workload. Reject files/drop-ins/transient or active units.
  test "$#" = 6 || return 1
  test "$1" = loaded && test "$2" = inactive && test -z "$3" &&
    test -z "$4" && test "$5" = no && test -z "$6"
}

check_browser_budget_host() {
  local policy="$1" output="$2" token sample driver version load active fragment dropins transient group
  local state_accepted=false path_absent=false accepted=false
  case "$policy" in unchanged) return 0 ;; high-1536m) ;; *) return 1 ;; esac
  test "${GITHUB_ACTIONS:-}" = true || return 1
  test "$output" = "$RUNNER_TEMP/reader-soak-report" || return 1
  test ! -e "$output/MEMORY_HIGH_HOST_PREFLIGHT.json" || return 1
  driver=$(docker info --format '{{.CgroupDriver}}') || driver=''
  version=$(docker info --format '{{.CgroupVersion}}') || version=''
  token=$(python3 -c 'import secrets; print(secrets.token_hex(8))') || return 1
  [[ "$token" =~ ^[0-9a-f]{16}$ ]] || return 1
  sample="readerhigh${token}.slice"
  load=$(systemctl show "$sample" --property=LoadState --value) || load=''
  active=$(systemctl show "$sample" --property=ActiveState --value) || active=''
  fragment=$(systemctl show "$sample" --property=FragmentPath --value) || fragment=unknown
  dropins=$(systemctl show "$sample" --property=DropInPaths --value) || dropins=unknown
  transient=$(systemctl show "$sample" --property=Transient --value) || transient=''
  group=$(systemctl show "$sample" --property=ControlGroup --value) || group=unknown
  if require_unconfigured_slice_state "$load" "$active" "$fragment" "$dropins" "$transient" "$group"; then
    state_accepted=true
  fi
  if [[ ! -e "/sys/fs/cgroup/$sample" ]]; then path_absent=true; fi
  if [[ "$driver" = systemd && "$version" = 2 && "$state_accepted" = true && "$path_absent" = true ]]; then
    accepted=true
  fi
  # Only finite manager fields; no Reader, real data or systemd writes. Preserve
  # failures BEFORE expensive artifact downloads or creating the actual budget.
  jq -n --arg driver "$driver" --arg version "$version" --arg load "$load" --arg active "$active" \
    --argjson stateAccepted "$state_accepted" --argjson pathAbsent "$path_absent" --argjson accepted "$accepted" \
    '{preflightOnly: true, systemdWritesPerformed: false, policy: "high-1536m",
      dockerCgroupDriver: $driver, dockerCgroupVersion: $version, generatedSliceLoadState: $load,
      generatedSliceActiveState: $active, unconfiguredSliceStateAccepted: $stateAccepted,
      generatedSliceCgroupAbsent: $pathAbsent, accepted: $accepted}' \
    > "$output/MEMORY_HIGH_HOST_PREFLIGHT.json" || return 1
  test "$accepted" = true
}

prepare_browser_budget() {
  local policy="$1" output="$2" token
  case "$policy" in
    unchanged) return 0 ;;
    high-1536m) ;;
    *) echo 'Unknown explicit browser memory policy' >&2; return 1 ;;
  esac
  test "${GITHUB_ACTIONS:-}" = true || return 1
  test "$output" = "$RUNNER_TEMP/reader-soak-report" || return 1
  test "$(docker info --format '{{.CgroupDriver}}')" = systemd || return 1
  test "$(docker info --format '{{.CgroupVersion}}')" = 2 || return 1
  token=$(python3 -c 'import secrets; print(secrets.token_hex(8))') || return 1
  [[ "$token" =~ ^[0-9a-f]{16}$ ]] || return 1
  budget_slice="readerhigh${token}.slice"
  budget_anchor="readerhigh${token}.service"
  budget_cgroup="/sys/fs/cgroup/$budget_slice"
  test ! -e "$budget_cgroup" || return 1
  require_unconfigured_slice_state \
    "$(systemctl show "$budget_slice" --property=LoadState --value)" \
    "$(systemctl show "$budget_slice" --property=ActiveState --value)" \
    "$(systemctl show "$budget_slice" --property=FragmentPath --value)" \
    "$(systemctl show "$budget_slice" --property=DropInPaths --value)" \
    "$(systemctl show "$budget_slice" --property=Transient --value)" \
    "$(systemctl show "$budget_slice" --property=ControlGroup --value)" || return 1
  test "$(systemctl show "$budget_anchor" --property=LoadState --value)" = not-found || return 1
  # The random names were observed absent. Cleanup is armed before starting,
  # including partial setup failures; it never stops a pre-existing slice.
  budget_started=true
  sudo systemd-run --unit="$budget_anchor" --slice="$budget_slice" \
    --property=Type=oneshot --property=RemainAfterExit=yes /bin/true || return 1
  sudo systemctl set-property --runtime "$budget_slice" CPUQuota=200% \
    MemoryHigh=1610612736 MemoryMax=2147483648 MemorySwapMax=0 TasksMax=256 || return 1
  python3 scripts/report-browser-cgroup.py --cgroup-root "$budget_cgroup" \
    --require-no-swap --expected-memory-high=1610612736 \
    | tee "$output/MEMORY_HIGH_PRESTART.json" || return 1
  budget_args=(--cgroup-parent "$budget_slice")
}

verify_browser_budget_membership() {
  local cid="$1" output="$2" pid membership
  if [[ "$budget_started" != true ]]; then return 0; fi
  [[ "$cid" =~ ^[0-9a-f]{64}$ ]] || return 1
  test "$(docker inspect --format '{{.Id}}' "$cid")" = "$cid" || return 1
  test "$(docker inspect --format '{{.HostConfig.CgroupParent}}' "$cid")" = "$budget_slice" || return 1
  pid=$(docker inspect --format '{{.State.Pid}}' "$cid") || return 1
  [[ "$pid" =~ ^[1-9][0-9]*$ ]] || return 1
  test "$pid" -gt 1 || return 1
  membership=$(cat "/proc/$pid/cgroup") || return 1
  test "$membership" = "0::/$budget_slice/docker-$cid.scope" || return 1
  test -k "$output" && test -O "$output" || return 1
  test "$(stat -c %u "$output")" != 10001 || return 1
  jq -n --arg cid "$cid" --arg slice "$budget_slice" --arg membership "$membership" \
    --argjson pid "$pid" \
    '{containerId: $cid, slice: $slice, hostPid: $pid, membership: $membership,
      verifiedBeforeProbe: true, hostEvidenceDirectorySticky: true,
      hostEvidenceOwnerDiffersFromRuntimeUid: true, policy: "high-1536m"}' > "$output/MEMORY_HIGH_MEMBERSHIP.json"
}

observe_browser_budget() {
  local output="$1" filename="${2:-MEMORY_HIGH_FINAL.json}"
  if [[ "$budget_started" != true ]]; then return 0; fi
  case "$filename" in MEMORY_HIGH_FINAL.json|MEMORY_HIGH_AFTER_SOAK.json) ;; *) return 1 ;; esac
  test ! -e "$output/$filename" || return 1
  # Observe cumulative parent peak/events before removing the owned container.
  # In particular, high pressure is recorded; max/OOM/PID events still fail.
  python3 scripts/report-browser-cgroup.py --cgroup-root "$budget_cgroup" \
    --require-no-swap --expected-memory-high=1610612736 \
    | tee "$output/$filename"
}

remove_browser_budget() {
  local output="$1" populated=null result=0 inactive=false attempt
  if [[ "$budget_started" != true ]]; then return 0; fi
  [[ "$budget_slice" =~ ^readerhigh[0-9a-f]{16}\.slice$ ]] || return 1
  test "$budget_anchor" = "${budget_slice%.slice}.service" || return 1
  # Call only AFTER Docker removal. Never kill an unexpected surviving process
  # by blindly stopping a populated slice, even though its name is owned.
  if [[ -d "$budget_cgroup" ]]; then
    for attempt in $(seq 1 30); do
      populated=$(awk '$1 == "populated" {print $2}' "$budget_cgroup/cgroup.events")
      if [[ "$populated" = 0 ]]; then break; fi
      sleep 0.1
    done
  else
    populated=0
  fi
  if [[ "$populated" = 0 ]]; then
    if sudo systemctl stop "$budget_anchor" "$budget_slice"; then
      if [[ "$(systemctl show "$budget_anchor" --property=ActiveState --value)" = inactive &&
            "$(systemctl show "$budget_slice" --property=ActiveState --value)" = inactive ]]; then
        inactive=true
      else result=1; fi
    else result=1; fi
  else result=1; fi
  jq -n --arg slice "$budget_slice" --arg anchor "$budget_anchor" \
    --argjson populated "$populated" --argjson inactive "$inactive" \
    '{slice: $slice, anchor: $anchor, populatedAfterContainerRemoval: $populated,
      ownedUnitsInactive: $inactive, policy: "high-1536m"}' > "$output/MEMORY_HIGH_CLEANUP.json" || return 1
  return "$result"
}

if [[ "${BASH_SOURCE[0]}" = "$0" ]]; then
  [[ "$#" = 3 && "$1" = preflight ]] || exit 1
  check_browser_budget_host "$2" "$3"
fi
