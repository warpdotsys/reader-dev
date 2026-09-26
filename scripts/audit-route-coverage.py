#!/usr/bin/env python3
"""Inventory JAR route literals, restored bindings, and differential harness references.

This is a static inventory, not a claim that a referenced route has been fully tested.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import zipfile


ROOT = Path(__file__).resolve().parents[1]
ROUTE = re.compile(rb"/reader3/[A-Za-z0-9/_-]+")
SOURCE_BINDING = re.compile(
    r'router\.(get|post|route)\("(/reader3/[A-Za-z0-9/_-]+)"\)'
)
OLD_CENTER = {
    "/reader3/activateLicense",
    "/reader3/generateKeys",
    "/reader3/generateLicense",
    "/reader3/isLicenseValid",
}


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def jar_routes(path):
    with zipfile.ZipFile(path) as archive:
        class_bytes = archive.read("BOOT-INF/classes/com/htmake/reader/api/YueduApi.class")
    return {match.decode("ascii") for match in ROUTE.findall(class_bytes)}


def source_routes(path):
    bindings = {}
    for method, route in SOURCE_BINDING.findall(path.read_text(encoding="utf-8")):
        bindings.setdefault(route, set()).add(method.upper())
    return bindings


def harness_references(directory):
    references = {}
    for path in sorted(directory.glob("compare-*")):
        if not path.is_file() or path.suffix not in {".py", ".ps1"}:
            continue
        found = {match.decode("ascii") for match in ROUTE.findall(path.read_bytes())}
        for route in found:
            references.setdefault(route, []).append(path.name)
    return references


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--original", type=Path, default=ROOT / "reference/original/reader-pro-3.2.14.original.jar")
    parser.add_argument("--source", type=Path, default=ROOT / "src/main/java/com/htmake/reader/api/YueduApi.kt")
    parser.add_argument("--scripts", type=Path, default=ROOT / "scripts")
    parser.add_argument("--json", type=Path, default=ROOT / "reports/route-coverage-inventory.json")
    parser.add_argument("--markdown", type=Path, default=ROOT / "reports/ROUTE-COVERAGE-INVENTORY.md")
    args = parser.parse_args()
    original = jar_routes(args.original)
    bindings = source_routes(args.source)
    references = harness_references(args.scripts)
    rows = []
    for route in sorted(original | bindings.keys()):
        harnesses = references.get(route, [])
        non_browser = any("webview" not in name for name in harnesses)
        tier = ("legacy-center-migrated" if route in OLD_CENTER else
                "non-browser-harness-sample" if non_browser else
                "browser-harness-only" if harnesses else "no-differential-harness")
        rows.append({
            "path": route,
            "inOriginalJar": route in original,
            "restoredMethodsFromSource": sorted(bindings.get(route, [])),
            "differentialHarnessReferences": harnesses,
            "browserOnlyReference": bool(harnesses) and all("webview" in name for name in harnesses),
            "legacyLicenseCenterRoute": route in OLD_CENTER,
            "coverageTier": tier,
        })
    no_harness = [row for row in rows if row["inOriginalJar"] and not row["differentialHarnessReferences"]]
    non_browser_harness = [row for row in rows if row["inOriginalJar"] and any(
        "webview" not in name for name in row["differentialHarnessReferences"]
    )]
    report = {
        "originalJarSha256": sha256(args.original),
        "sourceSha256": sha256(args.source),
        "counts": {
            "originalJarRouteLiterals": len(original),
            "restoredSourceRoutePaths": len(bindings),
            "originalRoutesAbsentFromRestoredSource": len(original - bindings.keys()),
            "originalRoutesReferencedByNonBrowserDifferentialHarness": len(non_browser_harness),
            "originalRoutesWithoutDifferentialHarnessReference": len(no_harness),
            "originalRoutesWithoutHarnessExcludingMigratedOldCenter": len([
                row for row in no_harness if not row["legacyLicenseCenterRoute"]
            ]),
        },
        "rows": rows,
        "dynamicRoutesOutsideYueduApi": [
            {"path": "/reader3/webdav/*", "note": "Registered by WebdavController; separate 19-probe lifecycle differential exists."}
        ],
        "limitations": [
            "路径取自 YueduApi 字节码和源码；字节码中存在字面量不能单独证明已注册路由或对应 HTTP 方法。",
            "脚本引用只是样本证据，不等于接口所有输入、状态和 HTTP 方法都已验证。",
            "不计 YueduApi 外的动态路由、前端资源和内部方法。",
            "本静态清单不认证最终构建；同一 JAR 的二十一套本地差分另见 UNIFIED-LOCAL-DIFFERENTIAL.md。",
        ],
    }
    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    lines = [
        "# 原 JAR 路由与差分脚本覆盖清单",
        "",
        "此表由 `python -B scripts/audit-route-coverage.py` 从原 JAR 的 `YueduApi.class`、恢复版路由源码及 `compare-*` 脚本生成。",
        "**脚本引用不等于已验证**；每个接口的行为结论仍以对应差分报告及逐项断言为准。",
        "",
        f"- 原 JAR 路径字面量：{len(original)}；恢复源码路径：{len(bindings)}。",
        f"- 原 JAR 路径中被非浏览器差分脚本直接引用：{len(non_browser_harness)}；完全没有差分脚本引用：{len(no_harness)}。",
        f"- 除去刻意迁移的 4 条旧中心路由，仍有 {len([row for row in no_harness if not row['legacyLicenseCenterRoute']])} 条原 JAR 路径没有差分脚本引用。",
        f"- 原 JAR 有而恢复源码没有：{', '.join(sorted(original - bindings.keys())) or '无'}。这四条为旧许可证中心路由，新中心不承诺原 JAR 等价。",
        "- `WebdavController` 动态注册的 `/reader3/webdav/*` 不计入上述 `YueduApi` 数量；其独立 19 项差分见 `WEBDAV-LIFECYCLE-DIFF.md`。",
        "- 原 JAR SHA-256：`" + report["originalJarSha256"] + "`；源码 SHA-256：`" + report["sourceSha256"] + "`。",
        "",
        "| 原 JAR 路径 | 恢复版方法（源码） | 证据层级 | 差分脚本中的直接引用 |",
        "| --- | --- | --- | --- |",
    ]
    for row in rows:
        suffix = "（旧中心，刻意迁移）" if row["legacyLicenseCenterRoute"] else ""
        lines.append("| `{}`{} | {} | {} | {} |".format(
            row["path"], suffix,
            ", ".join(row["restoredMethodsFromSource"]) or "—",
            row["coverageTier"],
            ", ".join("`" + name + "`" for name in row["differentialHarnessReferences"]) or "—",
        ))
    lines.extend(["", "## 边界", ""] + ["- " + item for item in report["limitations"]] + [""])
    args.markdown.write_text("\n".join(lines), encoding="utf-8")
    print(json.dumps(report["counts"], ensure_ascii=False))
    print(args.markdown)


if __name__ == "__main__":
    main()
