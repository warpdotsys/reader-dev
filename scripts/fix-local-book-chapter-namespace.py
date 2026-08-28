#!/usr/bin/env python3
"""修复本地书章节命名空间错位（BUGFIX: 本地书章节不存在）。

背景: v6.0.45 及更早版本多个章节写入路径未填充 book_chapters.user_namespace，
落库为 'default'；而读取路径严格按用户命名空间过滤。secure 模式下非 default
账号导入的本地书因此读不到正文（TOC 能列出，正文报「本地书章节不存在」）。
本脚本修复**既有数据**——把 user_namespace='default' 但归属书属于非 default
命名空间的章节行，修正为书所属命名空间。

用法:
  python3 scripts/fix-local-book-chapter-namespace.py [--db path/to/reader.db]
       [--dry-run] [--yes]

环境: 需先停止 reader-dev 服务（SQLite 写锁/备份一致性），建议先备份库再执行。
输出: 打印修正的章节行数（--dry-run 只统计不修改，exit 0）。
注意: 幂等——已修正的行不会再次匹配；default 命名空间书、非 default 已正确
      归属的章节均不受影响。
"""
import argparse
import sqlite3
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_DB = ROOT / "storage" / "reader.db"

# 相关子查询消歧：共享 book_url 归属多个命名空间时，确定性取字典序最小 ns。
# EXISTS 限定只处理“归属书存在非 default 命名空间”的章节，default 书不受影响。
FIX_SQL = """
UPDATE book_chapters
SET user_namespace = (
    SELECT user_namespace FROM books
    WHERE books.book_url = book_chapters.book_url
    ORDER BY user_namespace LIMIT 1
)
WHERE book_chapters.user_namespace = 'default'
  AND EXISTS (
    SELECT 1 FROM books
    WHERE books.book_url = book_chapters.book_url
      AND books.user_namespace <> 'default'
  )
"""

COUNT_SQL = """
SELECT COUNT(*) FROM book_chapters bc
WHERE bc.user_namespace = 'default'
  AND EXISTS (
    SELECT 1 FROM books b
    WHERE b.book_url = bc.book_url AND b.user_namespace <> 'default'
  )
"""

# 受影响章节按归属命名空间分组（dry-run 明细）
GROUP_SQL = """
SELECT b.user_namespace, COUNT(*)
FROM book_chapters bc
JOIN books b ON b.book_url = bc.book_url
WHERE bc.user_namespace = 'default'
  AND b.user_namespace <> 'default'
GROUP BY b.user_namespace
ORDER BY b.user_namespace
"""


def main() -> int:
    parser = argparse.ArgumentParser(
        description="修复本地书章节 user_namespace 错位（先备份库再执行）"
    )
    parser.add_argument("--db", type=Path, default=DEFAULT_DB,
                        help=f"reader.db 路径（默认 {DEFAULT_DB}）")
    parser.add_argument("--dry-run", action="store_true",
                        help="只统计待修复章节，不修改数据")
    parser.add_argument("--yes", action="store_true",
                        help="跳过执行前确认（建议仍先手动备份）")
    args = parser.parse_args()

    db_path = args.db.expanduser()
    if not db_path.exists():
        print(f"错误: 数据库不存在 {db_path}", file=sys.stderr)
        return 1

    conn = sqlite3.connect(str(db_path), timeout=30)
    try:
        pending = conn.execute(COUNT_SQL).fetchone()[0]
        print(f"待修复章节: {pending} 行（user_namespace='default' 但归属非 default 书）")
        if pending == 0:
            print("无需修复（数据库已正确或本 bug 不影响本库）")
            return 0

        if args.dry_run:
            print("\n受影响章节按归属命名空间分组（dry-run，未修改）:")
            for ns, cnt in conn.execute(GROUP_SQL).fetchall():
                print(f"  {ns}: {cnt} 行")
            print("dry-run 完成，未修改任何数据")
            return 0

        if not args.yes:
            answer = input(
                f"将修改 {db_path} 中 {pending} 行章节——"
                "执行前请确认已备份该库。继续? [y/N] "
            )
            if answer.strip().lower() not in ("y", "yes"):
                print("已取消，未修改任何数据")
                return 0

        rows = conn.execute(FIX_SQL).rowcount
        conn.commit()
        print(f"已修正 {rows} 行章节的 user_namespace")
        print("提示: 重新启动 reader-dev 后，既有本地书正文即可直接读取，无需重新导入")
        return 0
    finally:
        conn.close()


if __name__ == "__main__":
    raise SystemExit(main())