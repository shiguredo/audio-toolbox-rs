# CHANGES.md / SKILL.md に残っている issue 番号 / issue ファイル名の言及を除去する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/update-remove-issue-references-from-docs
- Polished: 2026-09-05

## 目的

shiguredo-issues 規約は issue 番号を書いてよい場所を `issues/` 配下のファイル名・`issues/SEQUENCE` ファイル・git コミットメッセージ・GitHub の PR 本文 / PR コメント / GitHub Issues コメントに限定し、リポジトリに残るドキュメント類 (README / docs / 設計書など)・ソースコード・`CHANGES.md` に issue 番号や issue への言及を書くことを禁じている。現状 `CHANGES.md` と `SKILL.md` にこれに反する記述があるため除去する。

## 優先度根拠

Medium とする。動作影響はないが、shiguredo-issues 規約の必守項目に反しているため見過ごせない。CHANGES.md の該当記述はリリース済みの 2026.1.0 セクションに既に載っており、issue 0027 の include 変更後は crates.io 配布物にも含まれるため、本 issue は 0027 より先に (または同一リリース内で) 完了させる。issue 0021 (CHANGES.md の develop セクション更新) とは対象セクションが異なるため順序は問わない。

## 現状

- `CHANGES.md` の `## 2026.1.0 / ### misc` の [UPDATE] `DECODE_BUF_FRAMES` コメント更新エントリの下位項目に以下の記述がある (issue ファイル名を含み、規約違反)。
  ```
  `issues/0002-investigate-decoder-output-buffer-vs-codec-limits.md` に調査内容を追記する
  ```
  なお、0002 の調査は完了しており調査内容は `issues/closed/0002-*` に記録済み (2026-04-01) で、この一文だけがリリース済みセクションに残っている。
- `skills/shiguredo-audio-toolbox/SKILL.md` の `Decoder::decode` の 1 パケット制約節に以下の記述がある (issue 番号 `0012` と GitHub PR リンクを含み、規約違反)。
  ```
  これは過去にあったバグ ([0012 で修正済み](https://github.com/shiguredo/audio-toolbox-rs/pull/4)) への対策。
  ```

## 完了条件

- `CHANGES.md` から上記 1 行 (issue ファイル名を含む記述) が削除される。エントリ本体 ([UPDATE] `DECODE_BUF_FRAMES` コメント更新の記述と担当者行) は残す。
- `SKILL.md` の上記文から issue 番号 / GitHub PR リンクが除去され、「過去にあったバグへの対策」として設計背景が issue 番号なしで残る。
- `issues/` と `.git/` を除くリポジトリ全体 (CHANGES.md / SKILL.md / ソースコード・コメント・テスト・docstring・README 等のドキュメント) に issue 番号 / issue ファイル名 / PR リンクの記述が無いことを、解決方法に示す grep パターンで確認する (検出された場合は本 issue の範囲に含めて除去する)。
- issue 番号 / issue ファイル名を新規に含めない。

## 解決方法

- `CHANGES.md` の該当 1 行のみを削除する。「調査内容を追記する」を散文に書き換える案は、規約が issue 番号だけでなく issue への言及そのものを書くことを禁じているため採らない。
- `SKILL.md` の該当文を以下のように書き換える (前段の「…制御している」の文はそのまま。事実関係は `issues/closed/0012-*` の記録に基づく):
  ```
  これは過去に、コールバックが同じパケットを複数回提供していたバグへの対策。
  ```
- issue 0014 が `SKILL.md` の同節 (1 パケット制約) にエラー時挙動の追記を行う予定のため、0014 を先に実施し、本 issue は 0014 の追記後に着手する。0014 の追記文は残し、issue 参照を含む文のみを書き換える。なお issue 0016 は「利用上の注意」セクションの別節への新規追加であり、本 issue の対象文の特定には影響しない。
- grep でリポジトリ全体 (`issues/` と `.git/` を除く。隠しファイルも対象) をスキャンする。パターン例:
  ```
  rg -n "issues/[0-9]{4,}|github\.com/[^ )]+/(pull|issues)/|#[0-9]+|0[0-9]{3,}" --hidden --glob '!issues/**' --glob '!.git/**' --glob '!Cargo.lock'
  ```
  examples/sine_to_mp4.rs のビット列コメント (00010 等)・Cargo.lock の checksum・`.github/workflows/*.yml` の commit SHA ピンは issue 番号と無関係のため目視で除外する。検出された場合は本 issue の範囲に含めて除去する。
