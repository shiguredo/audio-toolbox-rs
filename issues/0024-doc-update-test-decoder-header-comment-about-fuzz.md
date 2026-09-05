# tests/test_decoder.rs 冒頭の cargo-fuzz に関する虚偽コメントを修正する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/update-test-decoder-header-comment
- Polished: 2026-09-05

## 目的

`tests/test_decoder.rs` 冒頭のモジュールコメントが「圧縮バイト列の任意入力でのクラッシュ耐性は cargo-fuzz で扱う」と保証しているが、実際には fuzz が未整備で保証されていないため、この虚偽記述を修正する。

## 優先度根拠

Medium とする。動作に影響はないが、読者・後任のレビュアーに「fuzz でカバーされている」と誤誘導し、単体テストの追加判断や regression 分析を誤らせる。

## 現状

`tests/test_decoder.rs` 冒頭のモジュールコメント (ファイル先頭の `//!` コメント):

```rust
//! `Decoder` の単体テスト（`src/lib.rs` のデコーダー部分に対応）
//!
//! 圧縮バイト列の任意入力でのクラッシュ耐性は cargo-fuzz で扱う。
//! 共有定数・ファクトリは `include!("include/helpers.rs");` で取り込む。
```

しかしリポジトリに `fuzz/` ディレクトリが存在せず、ソースコード・テストに `fuzz_target` は 0 件。CHANGES.md 2026.1.0 misc では「`tests/test_decoder.rs` を単体テストのみとし、`proptest` を dev-dependencies から削除する」と記録済みで、クラッシュ耐性テストは実質的に無い状態。

## 完了条件

- `tests/test_decoder.rs` 冒頭コメントの虚偽行が解決方法の文言に書き換わり、その他の行は維持される。`grep -n "cargo-fuzz で扱う" tests/test_decoder.rs` が 0 件になり、`grep -n "現状未整備" tests/test_decoder.rs` が 1 件になる。
- `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

冒頭コメントの虚偽行 (「圧縮バイト列の任意入力でのクラッシュ耐性は cargo-fuzz で扱う。」) を以下の文言に書き換える (その他の行は維持する):

```
クラッシュ耐性テスト（cargo-fuzz）とプロパティテスト（noprop）は本ファイルの対象外（現状未整備）。将来導入する場合は本テストと重複しないよう役割を分担する。
```

「現状未整備」は時点依存の表現のため、将来 fuzz / プロパティテスト（noprop）を導入する時点で本コメントの該当部分も更新する。プロパティテストのツール名は shiguredo-rust 規約（PBT は noprop を使うこと）に従う。

issue 0023 と同時期に対応する (Makefile と tests/test_decoder.rs のコード変更は独立しているため順序は問わない)。ただし CHANGES.md の develop / `### misc` は 0021 / 0023 と重複するため、マージ時にコンフリクトした場合は develop の最新を取り込んで解決する。issue 0018 (テスト移設) と issue 0035 (include 行の書き換え) は本ファイルに触れるため、着手時に現状を確認する。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: [UPDATE] `tests/test_decoder.rs` 冒頭の cargo-fuzz に関する虚偽コメントを修正する)。

fuzz / プロパティテスト（noprop）を実際に導入する場合は、本 issue とは別に機能追加のカテゴリの issue を立てる。
