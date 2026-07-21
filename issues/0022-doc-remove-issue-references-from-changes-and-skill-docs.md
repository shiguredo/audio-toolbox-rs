# CHANGES.md / SKILL.md に残っている issue 番号 / issue ファイル名の言及を除去する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-remove-issue-references-from-docs
- Polished:

## 目的

shiguredo-issues 規約は issue 番号を書いてよい場所を `issues/` 配下・`SEQUENCE`・git コミットメッセージ・GitHub の PR/Issue 本文に限定し、リポジトリに残るドキュメント類 (README / CHANGES / SKILL / 設計書) に issue 番号を残すことを明示的に禁じている。現状 `CHANGES.md` と `SKILL.md` にこれに反する記述があるため除去する。

## 優先度根拠

Medium とする。動作影響はないが、shiguredo-issues 規約の必守項目に反しているため見過ごせない。CHANGES.md の記述は次回リリース時に露出する可能性があるので、issue 0021 (CHANGES.md 更新) と同時期に対応するのが自然。

## 現状

- `CHANGES.md:87` の `## 2026.1.0 / ### misc` に以下の記述がある。
  ```
  `issues/0002-investigate-decoder-output-buffer-vs-codec-limits.md` に調査内容を追記する
  ```
  issue ファイル名を含んでおり、規約違反。
- `skills/shiguredo-audio-toolbox/SKILL.md:240` の `Decoder::decode の 1 パケット制約` 節に以下の記述がある。
  ```
  これは過去にあったバグ ([0012 で修正済み](https://github.com/shiguredo/audio-toolbox-rs/pull/4)) への対策。
  ```
  issue 番号 (`0012`) と GitHub PR リンクを含んでおり、規約違反。

## 完了条件

- `CHANGES.md:87` から issue ファイル名参照が除去される (該当項目そのものを削除するか、issue ファイル名を含まない散文に書き換える)。
- `SKILL.md:240` から issue 番号 / GitHub PR リンクが除去される (「過去にあったバグへの対策」として設計背景を issue 番号なしで残す)。
- `CHANGES.md` / `SKILL.md` の他の箇所にも同種の記述が無いことを grep 等で確認する。

## 解決方法

`CHANGES.md:87` の該当エントリは、2026.1.0 は既にリリース済みだが遡って以下のいずれかに書き換える:

- 該当エントリ自体を削除する (`DECODE_BUF_FRAMES` のコメントを RFC 6716 §2.1.4 に基づいて明示する内容は残す)。
- または「調査内容を issue に追記する」の一文を削除し、コメント更新の事実だけを残す。

`SKILL.md:240` は以下のように書き換える (issue 番号なし):

- 「これは同一の AudioConverterFillComplexBuffer 呼び出し内で同じパケットが複数回コールバックに渡される Apple 側実装の挙動への対策。」

grep でリポジトリ全体を再スキャンし、他に issue 番号 / issue ファイル名 / PR リンクを含む記述が残っていないかも確認する。ソースコード本体・docstring・コメント・テスト名・テストコメント・その他ドキュメントも同時にチェックする。
