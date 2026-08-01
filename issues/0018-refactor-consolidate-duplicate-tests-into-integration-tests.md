# src/lib.rs::tests と tests/test_*.rs で重複しているテストを統合テスト側に一本化する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-consolidate-duplicate-tests
- Polished: 2026-07-31

## 目的

`src/lib.rs::tests` と `tests/test_*.rs` に重複したテストが多数存在する状態を解消し、公開 API のテストは `tests/` に一本化する。private を対象とする単体テストが必要になった時点で `src/<module>.rs` 内に単体テストを新設する運用に整理する。

## 優先度根拠

Medium とする。動作上は問題がないが、修正時の二重管理を強いること、shiguredo-rust 規約「private を対象とする単体テストは `src/<module>.rs` 内の `#[cfg(test)]` モジュールに書くこと」の趣旨に照らし、公開 API のみを対象とするテストが `src/lib.rs::tests` に残り `tests/` 側と重複する状態は規約と整合していないため、遠くない時期に片付けたい負債。

## 現状

`src/lib.rs` の `#[cfg(test)] mod tests` は以下のテストを持つ。

- `init_encoder`
- `encode_silent`
- `decode_aac_silent`
- `init_decoder_mp3`
- `init_decoder_opus`
- `test_supported_codecs`

これらは全て公開 API (`Encoder::new` / `Decoder::new` / `supported_codecs`) しか触っていない。

- `init_decoder_mp3` / `init_decoder_opus` は `tests/test_decoder.rs` の同名テストと **完全に一致** する。
- `test_supported_codecs` は `tests/test_codec_info.rs` の 4 テスト (`supported_codecs_has_expected_length_and_unique_codec_entries` / `supported_codecs_aac_lc_decode_and_encode` / `supported_codecs_mp3_decode_only_typically` / `supported_codecs_opus_decoding_supported`) のアサーションで `test_supported_codecs` のアサーションを全て包含する (一意性検証は統合テスト側が追加で行っている)。
- `init_encoder` の 3 分岐 (正常 128000 / 未指定 / 無効 1000) のうち、未指定 / 無効 1000 は `tests/test_encoder.rs` の `encoder_new_accepts_default_bitrate` / `encoder_new_rejects_invalid_bitrate` と重複し、正常 128000 は `encoder_new_accepts_each_bitrate_control_mode` 等で暗黙にカバーされている。
- `encode_silent` はエンコードのみのテスト、`decode_aac_silent` はエンコード → パケット化 → デコードの統合的なテストで、いずれも `tests/` 側には検証内容の等価物が無い (ただし `decode_aac_silent` のループ構造は `tests/test_decoder.rs` の `decode_multiple_aac_packets_no_duplicate_feeds` と重複するが、検証内容が異なるため統合対象としない)。公開 API しか触っていないため `tests/` に置くのが適切。

加えて、`src/lib.rs` の `encoder_config` ヘルパーは `tests/include/helpers.rs` の同名ヘルパーと関数シグネチャ・実装ともに一致しており、二重定義になっている。

## 完了条件

- `src/lib.rs::tests` モジュールが削除される (private を対象とする真の単体テストが必要になった時点で新設)。
- `tests/` 配下に `encode_silent` / `decode_aac_silent` 相当のテストが (helpers 経由で) 移設され、パスする (`decode_aac_silent` は「無音入力 → 全ゼロ出力」の PCM 内容検証を含む単独テストとして)。
- `src/lib.rs` 側の `encoder_config` 複製が消え、実体が `tests/` 側ヘルパーに一本化される。
- 既存の統合テストが引き続きパスする。
- 移設に伴い `tests/include/helpers.rs` のヘッダコメント (使用状況の記述) を更新する (現状のコメントも実態と一致していない箇所があるため、全ヘルパーの使用状況を実ファイルと突き合わせて書き直す。移設後は `tests/test_decoder.rs` でも `encoder_config` を使用する)。
- `CHANGES.md` の develop / `### misc` に [UPDATE] として追記する (追記エントリに issue 番号・issue ファイル名は含めない。issue 0022 の管轄)。

## 解決方法

1. `encode_silent` の内容を `tests/test_encoder.rs` に移設する。テスト名も `tests/` 側の命名規則 (`encoder_*` プレフィックス) に揃え、`expect` / `assert` メッセージは日本語で書く (英語メッセージのままコピーしない)。
2. `decode_aac_silent` の内容を `tests/test_decoder.rs` に移設する。「無音入力 → 全ゼロ出力」の PCM 内容検証は `decode_multiple_aac_packets_no_duplicate_feeds` に無い固有の検証のため、単独のテストとして移設し、統合はしない (テスト名は既存の `decode_*` 系に揃え、`expect` / `assert` メッセージは日本語で書く)。
3. `init_encoder` の 3 分岐 (正常 128000 / 未指定 / 無効 1000) は統合テスト側でカバー済みのため、追加テストは不要 (未指定 / 無効 1000 は `encoder_new_accepts_default_bitrate` / `encoder_new_rejects_invalid_bitrate` と完全重複、正常 128000 は `encoder_new_accepts_each_bitrate_control_mode` / `encoder_new_accepts_each_codec_quality` 等でカバー済み)。
4. `init_decoder_mp3` / `init_decoder_opus` / `test_supported_codecs` の削除。
5. 上記 1〜4 の移設・削除を済ませてから、`src/lib.rs` の `mod tests` 全体を削除する。
6. 最後に `cargo test` / `cargo fmt --all -- --check` / `cargo clippy --all-targets -- -D warnings` で確認する。
7. issue 0020 (`.unwrap()` を `.expect()` に置き換える) は、対象コードが本 issue で削除される `test_supported_codecs` 内にあり、0020 側も「0018 を先に片付ければ本 issue はクローズできる」と明記しているため、本 issue では作業を実施しない。本 issue の完了をもって 0020 は不要となりクローズする。

なお、テストのログメッセージ言語混在 (issue 0019) やヘルパー配置 (issue 0035) は別 issue で扱う。コールバック失敗パスの単体テスト (issue 0036) と MP3 エンコード対応アサーションの緩和 (issue 0037) は本 issue と対象・順序が絡むため、本 issue を先に実施し、その後に対応する (0036 のテスト追加先は `mod tests` を前提としない)。本 issue は「重複解消」に限定する。
