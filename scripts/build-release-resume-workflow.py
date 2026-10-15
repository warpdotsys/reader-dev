"""Deterministically derive recovery publishing/deployment from the canonical release jobs.

Default mode only checks committed bytes. --emit prints proposed YAML for review;
it never changes files. Business, registry verification, backup and rollback
commands come from release.yml; recovery changes only provenance/context binding.
"""
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[1]
HEADER='''name: Resume verified Reader release

on:
  workflow_dispatch:
    inputs:
      release_tag:
        description: Existing immutable stable tag, such as v6.1.0
        required: true
        type: string
      source_run_id:
        description: Failed original stable release run; all source and pressure gates must have passed
        required: true
        type: string

permissions:
  contents: read
  actions: read

concurrency:
  group: reader-pro-production
  cancel-in-progress: false

jobs:
  verify-resume-inputs:
    name: Verify immutable source artifacts before resuming publication
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    outputs:
      tag: ${{ steps.verify.outputs.tag }}
      version: ${{ steps.verify.outputs.version }}
      revision: ${{ steps.verify.outputs.revision }}
      source-run: ${{ steps.verify.outputs.source-run }}
      ghcr-image: ${{ steps.verify.outputs.ghcr-image }}
      dockerhub-image: ${{ steps.verify.outputs.dockerhub-image }}
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
        with:
          fetch-depth: 0
      - name: Require every original source gate and forbid an already activated release
        id: verify
        env:
          GH_TOKEN: ${{ github.token }}
          RELEASE_TAG: ${{ inputs.release_tag }}
          SOURCE_RUN: ${{ inputs.source_run_id }}
          GHCR_IMAGE: ${{ vars.GHCR_IMAGE }}
          DOCKERHUB_IMAGE: ${{ vars.DOCKERHUB_IMAGE }}
        run: python3 scripts/verify-release-resume-inputs.py --tag "$RELEASE_TAG" --source-run "$SOURCE_RUN" --output release-resume-inputs.json
      - name: Refuse a configured registry without reported push access before downloading images
        env:
          DOCKERHUB_USERNAME: ${{ secrets.DOCKERHUB_USERNAME }}
          DOCKERHUB_PASSWORD: ${{ secrets.DOCKERHUB_PASSWORD }}
          DOCKERHUB_IMAGE: ${{ vars.DOCKERHUB_IMAGE }}
        run: |
          python3 scripts/check-registry-publish-access.py --output registry-access.json
          python3 -c 'import json; value=json.load(open("registry-access.json")); assert value["completed"] and value["observations"]["configured"]["push"] is True'
      - name: Preserve bounded recovery input and grant evidence
        if: always()
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4
        with:
          name: release-resume-inputs-${{ github.run_id }}-${{ github.run_attempt }}
          path: |
            release-resume-inputs.json
            registry-access.json
          if-no-files-found: error
          retention-days: 90

'''

def replace(value,old,new,count):
    if value.count(old)!=count:raise ValueError('Canonical release shape changed: '+old[:100])
    return value.replace(old,new)

def render(canonical):
    first=canonical.index('  build-and-publish-images:\n')
    middle=canonical.index('  deploy-production:\n',first)
    final=canonical.index('  publish-github-release:\n',middle)
    publisher=canonical[first:middle]
    deploy=canonical[middle:final]
    github=canonical[final:]
    publisher=replace(publisher,'needs: [verify-release-inputs, verify-vue3-e2e, build-native-images]',
        'needs: verify-release-inputs',1)
    publisher=replace(publisher,'          digest-mismatch: error\n',
        '          digest-mismatch: error\n          github-token: ${{ github.token }}\n'
        '          run-id: ${{ needs.verify-release-inputs.outputs.source-run }}\n',4)
    publisher=replace(publisher,'      contents: read\n','      contents: read\n      actions: read\n',1)
    github=replace(github,'      contents: write\n','      contents: write\n      actions: read\n',1)
    deploy=replace(deploy,'          rollback_dir="$release_dir/rollback"\n',
        '          rollback_dir="$release_dir/rollback"\n'
        '          # Recovery must never replace an earlier snapshot for the same tag.\n'
        '          test ! -e "$rollback_dir/state-before-${RELEASE_TAG}.tar"\n',1)
    sections=[]
    for value,timeout in ((publisher,80),(deploy,30),(github,15)):
        value=replace(value,f'    timeout-minutes: {timeout}\n',f'    timeout-minutes: {timeout}\n'
            '    env:\n'
            '      RELEASE_TAG: ${{ needs.verify-release-inputs.outputs.tag }}\n'
            '      RELEASE_REVISION: ${{ needs.verify-release-inputs.outputs.revision }}\n',1)
        value=replace(value,'      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2\n',
            '      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2\n'
            '        with:\n          ref: ${{ needs.verify-release-inputs.outputs.tag }}\n',1)
        value=value.replace('verify-release-inputs','verify-resume-inputs')
        value=value.replace('${{ github.sha }}','${{ needs.verify-resume-inputs.outputs.revision }}')
        value=value.replace('${{ github.ref_name }}','${{ needs.verify-resume-inputs.outputs.tag }}')
        value=value.replace('$GITHUB_SHA','$RELEASE_REVISION').replace('$GITHUB_REF_NAME','$RELEASE_TAG')
        sections.append(value)
    return HEADER+''.join(sections)

def main():
    expected=render((ROOT/'.github/workflows/release.yml').read_text(encoding='utf-8'))
    if sys.argv[1:]==['--emit']:
        print(expected,end='');return
    if sys.argv[1:]:raise ValueError('Only --emit or default read-only check is supported')
    actual=(ROOT/'.github/workflows/release-resume.yml').read_text(encoding='utf-8')
    if actual!=expected:raise ValueError('Recovery workflow diverges from canonical release jobs; review and regenerate')
    print('Recovery publisher, deployment and rollback remain derived from the canonical release jobs')

if __name__=='__main__':main()
