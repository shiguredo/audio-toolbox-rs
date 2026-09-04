# CHANGES.md ## develop に反映されていない変更を追記する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/update-changes-md-develop-section
- Polished: 2026-09-04

## 目的

`CHANGES.md` の `## develop` セクションに反映されていない 2026.1.0 リリース以降の変更を追記し、次回リリース時点で `CHANGES.md` を利用者向けの正確な変更記録として提示できる状態にする。

## 優先度根拠

Medium とする。動作上の問題ではないが、次回リリース (2026.2.0) 前には必ず整えなければならない。shiguredo-changelog 規約に照らし、機能に直接影響しない変更は `### misc` に記載すべき。特に `Cargo.toml` の `include` 変更は crates.io に配布される中身が変わるため、利用者に必ず伝える必要がある。ただし公開 API の互換性には影響しないため、種別は `[CHANGE]` (後方互換のない変更) ではなく `### misc` の `[UPDATE]` で記載する。

## 現状

`CHANGES.md` の `## develop` セクションには issue 0012 の FIX 1 件のみが記載されており、以下のコミットが未反映 (日付昇順):

- `d4cc68e` (2026-05-20) GitHub Actions の外部 Action を commit SHA で固定する (misc。claude.yml は `b3129e7` で削除済みのため記載しない)
- `b3129e7` (2026-05-20) Claude Assistant ワークフローを削除する (misc)
- `c152a15` (2026-06-21) Cargo.toml を整理し依存ライブラリを更新する (misc。`authors` フィールド削除、`include` から `tests` 相当を除外 (crates.io 配布物の変更)、`Cargo.lock` の推移的依存 (bitflags / libc / regex 等) の更新。Cargo.toml の整理と Cargo.lock の依存更新の 2 エントリに分けて記載する)
- `8c8de56` (2026-07-17) prek.toml を shiguredo-rust 規約に合わせて整備する (misc。prek.toml と同時に `issues/closed/` 配下の .md も変更しているが、.md は対象外のため prek.toml のみ記載)
- `f5fe563` (2026-08-21) bindgen が生成する libc 関数宣言を除外して clippy を通す (misc。build.rs の変更で、生成されるバインディングから未使用の libc 関数 (memcmp / memcpy 等) が外れるが、公開 API には影響しない)

対象外の確定事項 (日付昇順):

- `a7f7f17` (2026-04-03) — 2026.1.0 リリース内容のマージのため対象外
- `58beff2` (2026-06-21) shiguredo-audio-toolbox スキルを追加する — `skills/shiguredo-audio-toolbox/SKILL.md` (.md) のみの変更のため、shiguredo-changelog 規約「.rst / .md ファイルの変更は変更履歴に反映しないこと」により対象外
- `243b4c7` (2026-06-21) AGENTS.md を更新する — .md のみの変更のため対象外
- `5a0f6ca` (2026-06-21) 0013 closed Encoder / Decoder のコールバック API 化を検討する — issue ファイル管理のため対象外
- `7e3ce83` (2026-06-21) [canary] Bump version — バージョン更新のみのため対象外 (CHANGES.md はリリース時に `## バージョン` セクションとして表現されるため)
- `abfba32` (2026-07-17) AGENTS.md に shiguredo-python スキル参照を追記する — .md のみの変更のため対象外
- `874c6f8` (2026-07-21) 0014-0038 open — issue ファイル管理のため対象外

加えて `## develop / ### misc` は見出しだけ残って中身が空。

## 完了条件

- `CHANGES.md ## develop` に上記の追記対象コミット (`c152a15` / `d4cc68e` / `b3129e7` / `8c8de56` / `f5fe563`) の変更が shiguredo-changelog 規約に沿って追記される。
- `Cargo.toml` の `include` 変更は `### misc` の [UPDATE] として明示される。
- 空の `### misc` セクションは中身が入るか削除される。
- issue 番号 / issue ファイル名を新規に含めない。
- shiguredo-changelog スキルを参照した上で分類とフォーマットが正しい。
- 対象は `## develop` セクションのみとし、他のセクション (2026.1.0 等) は変更しない。

## 解決方法

`## develop` セクションを以下のように書き直す (エントリの順序は現状セクションと同じ日付昇順とし、各エントリに担当者行 `- @voluntas` を付ける):

- 既存の `[FIX]` (Decoder callback 修正エントリ) はそのまま
- `### misc` に以下を列挙する (種別は [UPDATE]):
  - GitHub Actions の外部 Action を commit SHA で固定する変更 (`d4cc68e`)
  - Claude Assistant ワークフロー削除 (`b3129e7`)
  - `Cargo.toml` の整理 (`c152a15`。`authors` フィールド削除、配布物から `tests` 相当が外れた旨を利用者向けに書く。`include` の記述内容は issue 0027 により追加変更される可能性があるため、本エントリは `tests` 除外の事実のみを書く)
  - `Cargo.lock` の依存更新 (`c152a15`。bitflags / libc / regex 等の推移的依存の更新)
  - prek.toml を shiguredo-rust 規約に合わせて整備 (`8c8de56`)
  - bindgen が生成する libc 関数宣言を除外して clippy を通す (`f5fe563`。build.rs の `blocklist_function` 追加で生成バインディングから未使用の libc 関数が外れるが、利用者向けの挙動は変わらない旨を書く)

`c152a15` は上記の通り `Cargo.toml` の整理と `Cargo.lock` の依存更新の 2 エントリに分けて記載する。

書き加える際は shiguredo-changelog スキルを参照する。
