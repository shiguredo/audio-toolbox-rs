# tests/test_decoder.rs 冒頭の cargo-fuzz に関する虚偽コメントを修正する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-fix-test-decoder-header-comment
- Polished:

## 目的

`tests/test_decoder.rs` 冒頭のモジュールコメントが「圧縮バイト列の任意入力でのクラッシュ耐性は cargo-fuzz で扱う」と保証しているが、実際には fuzz が未整備で保証されていないため、この虚偽記述を修正する。

## 優先度根拠

Medium とする。動作に影響はないが、読者・後任のレビュアーに「Fuzz でカバーされている」と誤誘導し、単体テストの追加判断や regression 分析を誤らせる。issue 0023 (Makefile の未整備 fuzz ターゲット削除) と同じ根っこの問題で、同時に対応するのが自然。

## 現状

`tests/test_decoder.rs:1-4`:

```rust
//! `Decoder` の単体テスト（`src/lib.rs` のデコーダー部分に対応）
//!
//! 圧縮バイト列の任意入力でのクラッシュ耐性は cargo-fuzz で扱う。
//! 共有定数・ファクトリは `include!("include/helpers.rs");` で取り込む。
```

しかしリポジトリに `fuzz/` ディレクトリが存在せず、`grep -rn fuzz_target .` も 0 件。CHANGES.md 2026.1.0 misc では `proptest` を dev-dependencies から削除した経緯があり、クラッシュ耐性テストは実質的に無い状態。

## 完了条件

- `tests/test_decoder.rs` 冒頭コメントが実態と一致する記述に書き換わる。
- 併せて issue 0023 (Makefile の pbt / fuzzing ターゲット削除) と同時対応する。

## 解決方法

冒頭コメントを以下のいずれかに書き換える:

- 案 A (現状追認): 「クラッシュ耐性のテスト (proptest / cargo-fuzz) は現状未整備。将来導入する場合は本テストと重複しないよう役割を分担する。」
- 案 B (単体テストのみを目的化): 「本ファイルは公開 API に対する単体テストのみを扱う。」

案 A のほうが将来の整備方針を明示できるので推奨。案 B にする場合は将来 fuzz を追加する際に本コメントも更新する必要がある。

fuzz を実際に導入する意志がある場合は、本 issue とは別に `feature/add-fuzz-targets` などで立てる。
