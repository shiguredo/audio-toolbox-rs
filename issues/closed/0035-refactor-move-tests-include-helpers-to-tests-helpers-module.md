# tests/include/helpers.rs を tests/helpers/mod.rs に移し include! から mod 方式に切り替える

- Priority: Medium
- Created: 2026-07-21
- Completed: 2026-09-05
- Model: Opus 4.7
- Branch: feature/refactor-move-tests-helpers-to-mod
- Polished: 2026-07-31

## 目的

`tests/include/helpers.rs` を `include!("include/helpers.rs")` で各テストファイルに貼り付けている現状のパターンを、`tests/helpers/mod.rs` + `mod helpers;` の Rust 慣用形に切り替える。shiguredo-rust 規約「テスト間で共有するヘルパーは `tests/helpers/` に置くこと」と整合させる (取り込み方式は規約に規定がないため本 issue で `mod` 方式に統一する)。

## 優先度根拠

Medium とする。動作は正しいが、ヘルパーの配置が shiguredo-rust 規約に反している。`include!` 方式は各テスト binary で未使用になるヘルパーに `#[allow(dead_code)]` を付ける必要があるが、`mod` 方式でも各 binary で未使用になるヘルパーが残るため警告自体は消えない (実測済み)。代わりに `#[expect(dead_code)]` を `mod helpers;` 宣言に付けることで、shiguredo-rust 規約の「lint 警告を抑制する必要があるときは `#[allow(...)]` ではなく `#[expect(...)]` を使うこと」に整合させられる。

## 現状

- `tests/include/helpers.rs` の `encoder_config` / `decoder_config_aac` / `sine_pcm` / `encode_aac_packets` の 4 関数に `#[allow(dead_code)]` が付いている。
- `tests/test_decoder.rs` / `tests/test_encoder.rs` の先頭が `include!("include/helpers.rs")` でヘルパーを取り込んでいる。
- ヘッダコメント (`tests/include/helpers.rs` の先頭) の「クレート」表現が不正確 (実際は「統合テスト binary」)。`include!` 方式の記述と、どの binary でどのヘルパーが未使用かの記述も含む。
- `tests/test_codec_info.rs` はヘルパーを使用していない。

shiguredo-rust 規約はテスト補助を `tests/helpers/` に置くことと定めている。

## 完了条件

1. `tests/include/helpers.rs` が `tests/helpers/mod.rs` に移動し、`tests/include/` ディレクトリが存在しない (`test -f tests/helpers/mod.rs` が成功し、`test ! -e tests/include` が成功する)。
2. `tests/test_encoder.rs` / `tests/test_decoder.rs` の 2 ファイルの冒頭で `mod helpers;` として取り込み、`helpers::encoder_config(...)` のように qualified 参照で呼び出す (`grep -rn "include!" tests/` が 0 件)。`tests/test_codec_info.rs` はヘルパーを使用しないため対象外。
3. `#[allow(dead_code)]` が撤去され、`#[expect(dead_code, reason = "各テスト binary で使用されないヘルパーを許容する")]` が `mod helpers;` 宣言に付けられる (`grep -rn "allow(dead_code)" tests/` が 0 件、`grep -rn "expect(dead_code" tests/` が issue 0018 の実施状況に応じて 1〜2 件。0018 実施後は `test_decoder.rs` が全ヘルパーを使用するため expect は不要で 1 件になる)。
4. 既存の全テストが引き続きパスし、`cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` が成功する (ci.yml の clippy ステップ (`cargo clippy --workspace -- -D warnings`) はテストターゲットを含まないため、本 issue ではローカルで `--all-targets` 付きコマンドで検証する)。
5. ヘッダコメントの「クレート」表現と `include!` 記述が `tests/helpers/mod.rs` + `mod helpers;` の実態に合わせて修正される (更新後のコメントに `include!` の文字列が残らない文言にする。定数を含む全ヘルパーの使用状況を実ファイルと突き合わせて記述する。「クレート」→「テスト binary」の言い換えも行う)。各テストファイルの冒頭 doc コメントの `include!` 記述も `mod helpers;` 方式の記述に更新される。
6. `skills/shiguredo-audio-toolbox/SKILL.md` のソースファイル構成表の `tests/include/helpers.rs` 行が `tests/helpers/mod.rs` に更新される。
7. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない (issue 0022 の管轄)。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

1. `tests/include/helpers.rs` を `tests/helpers/mod.rs` に移動し、全関数・定数を `pub` にする (`mod helpers;` 経由で参照するため。private のままだと `error[E0603]` になる)。
2. `tests/test_encoder.rs` / `tests/test_decoder.rs` の冒頭を書き換える: `include!("include/helpers.rs");` を削除して `#[expect(dead_code, reason = "各テスト binary で使用されないヘルパーを許容する")] mod helpers;` を追加、ヘルパー呼び出しと定数参照を `helpers::encoder_config(...)` / `helpers::TEST_SAMPLE_RATE` のような qualified 参照に変更、冒頭 doc コメントの `include!` 記述を `mod helpers;` 方式の記述に更新。
3. `#[expect(dead_code)]` は `mod helpers;` 宣言に付ける (クレートルートに `#![expect(dead_code)]` を置くとテストファイル自身の将来のデッドコードも黙って抑制されるため。各 binary で未使用ヘルパーが存在するため expectation は必ず満たされる。関数ごとの `#[expect(dead_code)]` は使用側 binary で `unfulfilled_lint_expectations` 警告になり clippy が失敗するため使えない)。
4. `tests/include/` ディレクトリを削除する。
5. ヘッダコメントの「クレート」表現と `include!` 記述を `tests/helpers/mod.rs` + `mod helpers;` の実態に合わせて修正する (定数を含む全ヘルパーの使用状況を実ファイルと突き合わせて記述する。「クレート」→「テスト binary」の言い換えも行う。issue 0018 実施後は `test_decoder.rs` でも `encoder_config` を使用するため、使用状況の記述は 0018 の実施状況に合わせて更新する)。
6. `skills/shiguredo-audio-toolbox/SKILL.md` のソースファイル構成表の `tests/include/helpers.rs` 行を `tests/helpers/mod.rs` に更新する。
7. 最後に `cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` で確認する。

issue 0018 (テスト移設) / issue 0024 (`tests/test_decoder.rs` 冒頭コメント) / issue 0019 (テストメッセージ日本語化) が `tests/` 配下のファイルに触れるため、着手時に現状を確認し、マージ時にコンフリクトした場合は develop の最新を取り込んで解決する。issue 0018 実施後は `test_decoder.rs` でも `encoder_config` を使用するため、未使用ヘルパーが減って `#[expect(dead_code)]` が `unfulfilled_lint_expectations` になった場合は、該当テストファイルの expect を撤去する。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [UPDATE] テスト共有ヘルパーを tests/helpers/ に移し include! から mod 方式に切り替える`)。

**本 issue は対応せず closed にする**。`issues/0035-refactor-move-tests-include-helpers-to-tests-helpers-rs.md` (open, 2026-09-05 作成) と実質同一であり、後継版が本 issue の計画を置き換えているため。

- 重複の根拠: 同一 SEQ (0035)・同一目的 (`tests/include/helpers.rs` を include! から mod 方式に切り替える)・同一 Branch (`feature/refactor-move-tests-helpers-to-mod`)・本文の構成も同一で、違いは配置先の指定のみ。
- 陳腐化の根拠: 後継版は 2026-09-05 に open され `Polished: 2026-09-05` (当日磨き済み)。後継版の「`tests/helpers/mod.rs` (ディレクトリ方式) を採らない理由」は、shiguredo-rust 規約 (スキルの「`mod.rs` を使わないこと」、`src/` に限らず `tests/` や `examples/` でも同様) を引用し、単一ファイル `tests/helpers.rs` を採るとしている。本 issue の `tests/helpers/mod.rs` 案は同規約に反するため、このまま実装すると誤った変更が入る。
- 実測 (2026-09-05): `tests/include/helpers.rs` は存在しており `include!("include/helpers.rs")` 方式が維持されている (`tests/test_encoder.rs` / `tests/test_decoder.rs`)。`skills/shiguredo-audio-toolbox/SKILL.md` のソースファイル構成表にも `tests/include/helpers.rs` 行が残っている。つまり 0035 相当の実装は未着手であり、実施すべき後継は `issues/0035-refactor-move-tests-include-helpers-to-tests-helpers-rs.md` の 1 件に集約される。
- `Polished:` は更新しない (closed への遷移であり、auto-resolve 対象への誤昇格を避ける)。
