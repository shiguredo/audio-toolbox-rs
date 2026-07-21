# tests/include/helpers.rs を tests/helpers/mod.rs に移し include! から mod 方式に切り替える

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-move-tests-helpers-to-mod
- Polished:

## 目的

`tests/include/helpers.rs` を `include!("include/helpers.rs")` で各テストファイルに貼り付けている現状のパターンを、`tests/helpers/mod.rs` + `mod helpers;` の Rust 慣用形に切り替える。shiguredo-rust 規約「テスト間で共有するヘルパーは `tests/helpers/` に置くこと」と整合させる。

## 優先度根拠

Medium とする。動作は正しいが、`include!` 方式は per-file の複製を生成するため `#[allow(dead_code)]` を全ヘルパー関数に付ける必要があり、shiguredo-rust 規約の `#[allow(...)]` ではなく `#[expect(...)]` を使う原則にも影響する。`mod` 方式にすれば `#[allow(dead_code)]` は不要になる可能性が高い。

## 現状

- `tests/include/helpers.rs:10, 23, 33, 49` に `#[allow(dead_code)]` が 4 箇所付いている。
- `tests/test_decoder.rs:6` / `tests/test_encoder.rs:4` が `include!("include/helpers.rs")` でヘルパーを取り込んでいる。
- ヘッダコメント (`tests/include/helpers.rs:1-4`) の「クレート」表現が不正確 (実際は「統合テスト binary」)。

shiguredo-rust 規約はテスト補助を `tests/helpers/` に置くことを推奨し、`mod` 経由の取り込みを標準とする。

## 完了条件

- `tests/include/helpers.rs` が `tests/helpers/mod.rs` に移動する。
- 各 `tests/test_*.rs` の冒頭で `mod helpers;` として取り込み、`helpers::encoder_config(...)` のように呼び出す。
- `#[allow(dead_code)]` が撤去される (mod 方式では未使用関数警告が変わる可能性があるため確認)。
- 現行のテストが引き続きパスする。

## 解決方法

1. `tests/include/helpers.rs` を `tests/helpers/mod.rs` に移動する。
2. 各テストファイルの冒頭を書き換える:
   - `include!("include/helpers.rs");` を削除
   - `mod helpers;` を追加
   - ヘルパー呼び出しを `helpers::encoder_config(...)` などに変更
3. `TEST_SAMPLE_RATE` / `TEST_CHANNELS` / `AAC_FRAMES_PER_PACKET` などの定数は `helpers::TEST_SAMPLE_RATE` のように参照する。もしくは `use helpers::*;` を先頭で書く。
4. `#[allow(dead_code)]` を撤去して `cargo test --workspace` / `cargo clippy --all-targets -- -D warnings` を試す。未使用警告が出るなら `#[expect(dead_code)]` に変える。
5. `tests/include/` ディレクトリを削除する。
6. ヘッダコメントの「クレート」表現も修正する。

なお、issue 0018 (テスト重複解消) と同時期に対応するとテストコード全体が一度に整うが、目的が異なるため別 issue として管理する。
