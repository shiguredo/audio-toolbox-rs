# canary.py を shiguredo-python 規約準拠に書き直す

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-rewrite-canary-py
- Polished: 2026-07-31

## 目的

`canary.py` を shiguredo-python 規約 (uv 管理、`ruff` / `ty`、`from __future__ import annotations`、`str | None`、`logging`、`__all__` 等) に沿って全面的に書き直す。合わせて事故防止のガード (作業ディレクトリの検証、Cargo.toml パースの堅牢化) を入れる。docstring は AGENTS.md の「コメントはしっかり入れること」に基づき日本語で付ける。

## 優先度根拠

Medium とする。canary リリース用の補助スクリプトであり運用上は「これで動いてしまう」が、shiguredo-python 規約の複数項目に反しており規約整合として近い時期に片付ける必要がある。作業ディレクトリ未検証は、別リポジトリで叩いた場合にそのリポジトリを書き換え commit / tag / push し、タグ push が release.yml の発火経路であるため誤リリースに繋がりうる実害がある。プロンプトの `(Y/n)` 慣習逸脱は一般的な CLI 慣習 (大文字がデフォルト値) からの逸脱で、規約整合の観点で直す。

## 現状

`canary.py` に以下の問題がある。

- リポジトリに `pyproject.toml` / `uv.lock` が無く uv 管理外。`ruff` / `ty` の対象になっていない。
- `from __future__ import annotations` が無い。
- `from typing import Optional` を使い `Optional[str]` を宣言している (Python 3.12+ は `str | None`)。
- 関数に docstring が無い (AGENTS.md のコメント規約に反する)。
- `__all__` が無い。
- `logging` ではなく `print` を多用。
- `subprocess.run` の前に作業ディレクトリの検証や `git status --porcelain` の事前確認が無い (別リポジトリで叩く事故リスク)。
- `re.subn` で Cargo.toml を書き換えている (`tomllib` + `tomli-w` に置き換える。`tomli_w` はコメントを保持しないため、書き換え結果の検証が必要)。
- `input("Do you want to update the version? (Y/n): ")` の `(Y/n)` は一般的な CLI 慣習 (大文字がデフォルト値) と逆で、実装は空 Enter を cancel 扱い。
- 非 canary バージョンからの遷移はマイナー +1・パッチ 0・`-canary.0` を生成する正規表現置換 (実測: `2026.1.0` → `2026.2.0-canary.0`) だが、文字列置換のため堅牢性に欠ける。

加えて `.mypy_cache/` がリポジトリに存在するがルートの `.gitignore` に無く (mypy 自動生成の `.mypy_cache/.gitignore` による自己無視に依存している)、Python 環境が uv / ty に移行していない証左。

## 完了条件

1. `pyproject.toml` と `uv.lock` が新設され、`uv sync` が成功する (`ls pyproject.toml uv.lock` が成功する)。
2. `canary.py` に `from __future__ import annotations` が入り、型記法が `str | None` になる (`grep -nE "Optional|typing import" canary.py` が 0 件、`grep -n "from __future__ import annotations" canary.py` が 1 件)。
3. `canary.py` の全関数 (`update_version` / `run_cargo_update` / `git_commit_version` / `git_operations_after_build` / `main` / 新設の `next_version` 等) に日本語 docstring が付く (`grep -o '"""' canary.py | wc -l` が関数数の 2 倍以上)。
4. `print` が `logging` (英語メッセージ) に置き換わる (`grep -n "print(" canary.py` が 0 件)。
5. `main` 冒頭 (Cargo.toml への書き込み前) で、Cargo.toml の `[package] name` が `shiguredo_audio_toolbox` であること、および `git status --porcelain --untracked-files=no` が空であることを検証し、失敗時はエラーメッセージと非 0 終了コードで停止する (`grep -n "shiguredo_audio_toolbox" canary.py` が 2 件以上、`grep -n "git status" canary.py` が 1 件)。`--dry-run` 時は書き込みがないため git status ガードをスキップする。
6. Cargo.toml のバージョン更新が `tomllib` + `tomli-w` ベースになり、現行挙動 (canary あり: `-canary.N` の N を +1 / canary なし: マイナー +1・パッチ 0・`-canary.0` 付与) を維持する。git 操作 (commit メッセージ `[canary] Bump version to ...`・tag・push 順序) も現行を維持する。書き換え後に `tomllib` で再パースし、`[package].version` が期待値と一致することを確認する。
7. バージョン遷移ロジックの pytest テストが `tests/test_canary.py` に日本語コメント付きで追加され、`uv run pytest` が成功する。
8. プロンプトが `(y/N):` に変わり、空 Enter は N (cancel) として扱う。プロンプトは英語のままにする。
9. `.gitignore` に `.mypy_cache/` と `.venv/` が追加され、リポジトリに `.mypy_cache/` が残っていない (`grep -n "mypy_cache" .gitignore` が 1 件、`grep -n "\.venv" .gitignore` が 1 件、`ls .mypy_cache` が失敗する)。
10. `prek.toml` に `ruff format` / `ruff check` / `ty check` / `pytest` のフックが追加され、prek で通過する。tombi の lint / format 対象から `**/uv.lock` を除外する。
11. `canary.py` に `__all__` が追加される (`grep -n "__all__" canary.py` が 1 件以上)。
12. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

段階的にリファクタする (issue は 1 件のまま)。PR を分割する場合は、第 1 段 (完了条件 1〜3) と第 2 段 (完了条件 4〜12) で分け、各 PR で満たす完了条件を明記する。CI への Python ジョブ追加は行わない (補助スクリプトのため、ローカル検証は prek フックで担保する)。

1. `pyproject.toml` を新設し、ruff / ty の設定を書く (shiguredo-python スキル参照)。`uv lock` / `uv sync` で `uv.lock` を生成する。`tomli-w` は用途コメント付きで dev-dependencies に追加する。パッケージ化しない構成 (`package = false` 等) とし、pytest がルートの `canary.py` を import できる設定 (`pythonpath` 等) を含める。
2. `canary.py` の冒頭に `from __future__ import annotations` を追加し、`Optional[str]` を `str | None` に置換、`from typing import Optional` を削除。
3. 各関数に日本語 docstring を付ける (「バージョン更新」「cargo update 実行」「git コミット・タグ付け」等)。
4. `logging` モジュールを使い、`getLogger(__name__)` でロガーを取得。`print` を `logger.info` / `logger.error` (英語メッセージ) に置換。
5. `main` 冒頭 (Cargo.toml への書き込み前) で、`Cargo.toml` の `[package] name` が `shiguredo_audio_toolbox` であること、および `git status --porcelain --untracked-files=no` が空であることを検証し、失敗時は明確なエラーと非 0 終了コードで終了する。`--dry-run` 時は git status ガードをスキップする。
6. バージョン遷移ロジックを純関数 (例: `next_version(current_version: str) -> str`) に切り出し、プロンプト処理は `main` 側に置く。`update_version` の中身を `tomllib` と `tomli_w` で書き直し、現行挙動 (canary あり: `-canary.N` の N を +1 / canary なし: マイナー +1・パッチ 0・`-canary.0` 付与) と git 操作 (commit メッセージ・tag・push 順序) を維持する。
7. `next_version` の pytest テストを `tests/test_canary.py` に日本語コメント付きで追加する (canary あり / canary なし / バージョン未発見の 3 分岐)。
8. プロンプトを `(y/N):` に変えて空 Enter は N (cancel) として扱う。
9. `.gitignore` に `.mypy_cache/` と `.venv/` を追加し、既存の `.mypy_cache/` を削除する (git 管理下ではないため `git rm` は不要)。
10. prek に `ruff format` / `ruff check` / `ty check` / `pytest` のフックを追加し、tombi の lint / format 対象から `**/uv.lock` を除外する。
11. `canary.py` に `__all__` を追加する。
12. `CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: [UPDATE] canary.py を shiguredo-python 規約に合わせて書き直す)。

書き換え後に `uv run python canary.py --dry-run` で一連のフローを動作確認する (`--dry-run` 時は git status ガードをスキップする仕様のため、未コミット状態でも実行できる)。
