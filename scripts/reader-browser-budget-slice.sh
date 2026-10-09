#!/usr/bin/env bash
# Source only from the generated offline soak on a fresh GitHub-hosted runner.
# This is an opt-in parent policy, not an image/heap/font or production change.
budget_slice=''
budget_anchor=''
budget_cgroup=''
budget_started=false
budget_args=()

prepare_browser_budget() {
  local policy="$1" output="$2" token
  case "$policy" in
    unchanged) return 0 ;;
    high-1536m) ;;
    *) echo 'Unknown explicit browser memory policy' >&2; return 1 ;;
  esac
  test "${GITHUB_ACTIONS:-}" = true
  test "$output" = "$RUNNER_TEMP/reader-soak-report"
  test "$(docker info --format '{{.CgroupDriver}}')" = systemd
  test "$(docker info --format '{{.CgroupVersion}}')" = 2
  token=$(python3 -c 'import secrets; print(secrets.token_hex(8))')
  [[ "$token" =~ ^[0-9a-f]{16}$ ]]
  budget_slice="readerhigh${token}.slice"
  budget_anchor="readerhigh${token}.service"
  budget_cgroup="/sys/fs/cgroup/$budget_slice"
  test ! -e "$budget_cgroup"
  test "$(systemctl show "$budget_slice" --property=LoadState --value)" = not-found
  test "$(systemctl show "$budget_anchor" --property=LoadState --value)" = not-found
  # The random names were observed absent. Cleanup is armed before starting,
  # including partial setup failures; it never stops a pre-existing slice.
  budget_started=true
  sudo systemd-run --unit="$budget_anchor" --slice="$budget_slice" \
    --property=Type=oneshot --property=RemainAfterExit=yes /bin/true
  sudo systemctl set-property --runtime "$budget_slice" CPUQuota=200% \
    MemoryHigh=1610612736 MemoryMax=2147483648 MemorySwapMax=0 TasksMax=256
  python3 scripts/report-browser-cgroup.py --cgroup-root "$budget_cgroup" \
    --require-no-swap --expected-memory-high=1610612736 \
    | tee "$output/MEMORY_HIGH_PRESTART.json"
  budget_args=(--cgroup-parent "$budget_slice")
}

verify_browser_budget_membership() {
  local cid="$1" output="$2" pid membership
  if [[ "$budget_started" != true ]]; then return 0; fi
  [[ "$cid" =~ ^[0-9a-f]{64}$ ]]
  test "$(docker inspect --format '{{.Id}}' "$cid")" = "$cid"
  test "$(docker inspect --format '{{.HostConfig.CgroupParent}}' "$cid")" = "$budget_slice"
  pid=$(docker inspect --format '{{.State.Pid}}' "$cid")
  [[ "$pid" =~ ^[1-9][0-9]*$ ]]
  test "$pid" -gt 1
  membership=$(cat "/proc/$pid/cgroup")
  test "$membership" = "0::/$budget_slice/docker-$cid.scope"
  jq -n --arg cid "$cid" --arg slice "$budget_slice" --arg membership "$membership" \
    --argjson pid "$pid" \
    '{containerId: $cid, slice: $slice, hostPid: $pid, membership: $membership,
      verifiedBeforeProbe: true, policy: "high-1536m"}' > "$output/MEMORY_HIGH_MEMBERSHIP.json"
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
