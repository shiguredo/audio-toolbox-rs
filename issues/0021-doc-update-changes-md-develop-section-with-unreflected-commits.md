# CHANGES.md ## develop に反映されていない変更を追記する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/change-update-changes-md-develop-section
- Polished:

## 目的

`CHANGES.md` の `## develop` セクションに反映されていない 2026.1.0 リリース以降の変更を追記し、次回リリース時点で `CHANGES.md` を利用者向けの正確な変更記録として提示できる状態にする。

## 優先度根拠

Medium とする。動作上の問題ではないが、次回リリース (2026.2.0) 前には必ず整えなければならない。shiguredo-changelog 規約に照らし、`.rst` / `.md` の変更以外は `### misc` にでも記載すべき。特に `Cargo.toml` の `include` 変更は crates.io に配布される中身が変わるため、利用者に必ず伝える必要がある `[CHANGE]` 級のエントリ。

## 現状

`CHANGES.md:12-21` の `## develop` セクションには issue 0012 の FIX 1 件のみが記載されており、以下のコミットが未反映:

- `c152a15` (2026-06-21) Cargo.toml を整理し依存ライブラリを更新する
  - `authors` フィールド削除
  - `include` から `tests` 相当を除外 (crates.io 配布物の変更)
  - `dev-dependencies` のマイナーバージョン更新
- `d4cc68e` (2026-05-20) GitHub Actions の外部 Action を commit SHA で固定する (misc)
- `b3129e7` (2026-05-20) Claude Assistant ワークフローを削除する (misc)
- `58beff2` (?) shiguredo-audio-toolbox スキルを追加する (misc)
- `8c8de56` (2026-07-17) prek.toml を shiguredo-rust 規約に合わせて整備する (misc)
- `abfba32` 整備 (内容次第で misc か対象外)

加えて `CHANGES.md:20-21` の `## develop / ### misc` は見出しだけ残って中身が空。

なお、issue 0022 (issue 番号残存除去) と関連するため、`## develop` に追記するエントリでも issue 番号 / issue ファイル名を書かないよう注意する。

## 完了条件

- `CHANGES.md ## develop` に上記コミットの変更が shiguredo-changelog 規約に沿って追記される。
- `Cargo.toml` の `include` 変更は `[CHANGE]` として明示される。
- 空の `### misc` セクションは中身が入るか削除される。
- issue 番号 / issue ファイル名を新規に含めない。
- shiguredo-changelog スキルを参照した上で分類とフォーマットが正しい。

## 解決方法

`## develop` セクションを以下のように書き直す (順序は shiguredo-changelog 規約に従う):

- `[CHANGE]` として `Cargo.toml` の `include` 変更を記載する
  - 配布物から `tests` 相当が外れた旨を利用者向けに書く
- `[FIX]` として既存の Decoder callback 修正エントリはそのまま
- `### misc` に以下を列挙する:
  - GitHub Actions の外部 action を commit SHA で固定する変更
  - Claude Assistant ワークフロー削除
  - shiguredo-audio-toolbox スキル追加
  - prek.toml を shiguredo-rust 規約に合わせて整備
  - `abfba32` の内容 (git show で確認して該当するなら)
  - `dev-dependencies` の更新
- 上記のいずれかが `[CHANGE]` に該当するか (例えば prek 移行は開発者向けとして misc) を書き分ける

書き加える際は shiguredo-changelog スキルの必読と、issue 0022 (issue 番号残存の除去) の方針を先に読んでから作業する。
