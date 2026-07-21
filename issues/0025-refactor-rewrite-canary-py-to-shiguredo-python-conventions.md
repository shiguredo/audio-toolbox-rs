# canary.py を shiguredo-python 規約準拠に書き直す

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-rewrite-canary-py
- Polished:

## 目的

`canary.py` を shiguredo-python 規約 (uv 管理、`ruff` / `ty`、`from __future__ import annotations`、`str | None`、`logging`、docstring、`__all__` 等) に沿って全面的に書き直す。合わせて事故防止のガード (作業ディレクトリの検証、Cargo.toml パースの堅牢化) を入れる。

## 優先度根拠

Medium とする。canary リリース用の補助スクリプトであり運用上は「これで動いてしまう」が、shiguredo-python 規約の複数項目に反しており規約整合として近い時期に片付ける必要がある。また `(Y/n)` 慣習違反や作業ディレクトリ未検証は誤操作の実害を招きうる。

## 現状

`canary.py` に以下の shiguredo-python 規約違反がある。

- リポジトリに `pyproject.toml` / `uv.lock` が無く uv 管理外。`ruff` / `ty` の対象になっていない。
- `from __future__ import annotations` が無い。
- `from typing import Optional` を使い `Optional[str]` を宣言している (Python 3.12+ は `str | None`)。
- 関数に docstring が無い。
- `__all__` が無い。
- `logging` ではなく `print` を多用。
- `subprocess.run` の前に作業ディレクトリの検証や `git status --porcelain` の事前確認が無い (別リポジトリで叩く事故リスク)。
- `re.subn` で Cargo.toml を書き換えている (`tomllib` + `tomli-w` が堅い)。
- `input("Do you want to update the version? (Y/n): ")` で慣習違反 (`(Y/n)` の大文字は Enter だけで Y が慣習だが、実装は空 Enter を cancel 扱い)。

加えて `.mypy_cache/` がリポジトリに存在するが `.gitignore` に無く、Python 環境が uv / ty に移行していない証左。

## 完了条件

- `pyproject.toml` が新設され canary.py が uv 管理下に入る。
- `from __future__ import annotations` が入り、型記法が `str | None` になる。
- 関数に日本語 docstring が付く。
- `print` が `logging` に置き換わる。
- `subprocess.run` 前に作業ディレクトリ (`Cargo.toml` の存在) と `git status --porcelain` を検証する。
- Cargo.toml のバージョン更新が `tomllib` + `tomli-w` (もしくは `cargo set-version`) ベースになる。
- プロンプトが `(y/N):` に変わるか、実装で空 Enter を Y と扱う。
- `.gitignore` に `.mypy_cache/` を追加する。
- prek などから ruff / ty で lint できる状態になる。

## 解決方法

段階的にリファクタする。

1. `pyproject.toml` を新設し、ruff / ty の設定を書く (shiguredo-python スキル参照)。
2. `canary.py` の冒頭に `from __future__ import annotations` を追加。
3. `Optional[str]` を `str | None` に置換、`from typing import Optional` を削除。
4. 各関数に日本語 docstring を付ける (「バージョン更新」「cargo update 実行」「git コミット・タグ付け」等)。
5. `logging` モジュールを使い、`getLogger(__name__)` でロガーを取得。`print` を `logger.info` / `logger.error` に置換。
6. `main` 冒頭で `Path("Cargo.toml").exists()` と `git status --porcelain` の空を検証し、失敗時は明確なエラーで終了する。
7. `update_version` の中身を `tomllib` (Python 3.11+) と `tomli_w` で書き直す。あるいは `subprocess.run(["cargo", "set-version", ...])` に置換 (cargo-edit の導入が別途必要)。
8. プロンプトを `(y/N)` に変えて空 Enter は N として扱う、もしくは `(Y/n)` を維持して空 Enter を Y として扱う。日本語プロンプトも検討する。
9. `.gitignore` に `.mypy_cache/` を追加し、既存の `.mypy_cache/` を削除する。
10. prek に ruff / ty のフックを追加する (可能なら)。

対応工数が大きいので、上記を「pyproject.toml + `from __future__` + `str | None`」の第 1 段と「logging / ガード / TOML パーサ / プロンプト」の第 2 段に分けて PR にしてもよい (issue は 1 件のまま作業を分割)。
