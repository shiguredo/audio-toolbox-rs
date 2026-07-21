# src/lib.rs::tests と tests/test_*.rs で重複しているテストを統合テスト側に一本化する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-consolidate-duplicate-tests
- Polished:

## 目的

`src/lib.rs::tests` と `tests/test_*.rs` に重複したテストが多数存在する状態を解消し、公開 API のテストは `tests/` に一本化する。private を対象とする単体テストが必要になった時点で `src/<module>.rs` 内に単体テストを新設する運用に整理する。

## 優先度根拠

Medium とする。動作上は問題がないが、修正時の二重管理を強いること、shiguredo-rust 規約「`tests/`・`pbt/`・`fuzz/` のテストは公開 API に対してだけ書くこと」「private を対象とする単体テストは `src/<module>.rs` 内の `#[cfg(test)]` モジュールに書くこと」の裏返し (「公開 API のテストは `tests/` に」) に照らして整合していないため、遠くない時期に片付けたい負債。

## 現状

`src/lib.rs:1039-1188` の `#[cfg(test)] mod tests` は以下のテストを持つ。

- `init_encoder` (L1058)
- `encode_silent` (L1071)
- `decode_aac_silent` (L1092)
- `init_decoder_mp3` (L1137)
- `init_decoder_opus` (L1146)
- `test_supported_codecs` (L1157)

これらは全て公開 API (`Encoder::new` / `Decoder::new` / `supported_codecs`) しか触っていない。

- `init_decoder_mp3` / `init_decoder_opus` は `tests/test_decoder.rs:44-62` の同名テストと **完全に一致** する。
- `test_supported_codecs` は `tests/test_codec_info.rs:5-51` の 3 テスト (`supported_codecs_has_expected_length_and_unique_codec_entries` / `supported_codecs_aac_lc_decode_and_encode` / `supported_codecs_mp3_decode_only_typically` / `supported_codecs_opus_decoding_supported`) と実質的に同じアサーションを行う。
- `init_encoder` は `tests/test_encoder.rs:43-50` の `encoder_new_rejects_invalid_bitrate` / `encoder_new_accepts_default_bitrate` と重複する範囲を含む。
- `encode_silent` と `decode_aac_silent` はより本格的な encode/decode の統合的なテストで、`tests/` 側にはまだ等価物が無いが、公開 API しか触っていないため `tests/` に置くのが適切。

加えて、`src/lib.rs:1046-1056` の `encoder_config` ヘルパーは `tests/include/helpers.rs:11-21` と関数シグネチャ・実装ともに一致しており、二重定義になっている。

## 完了条件

- `src/lib.rs::tests` モジュールが削除される (private を対象とする真の単体テストが必要になった時点で新設)。
- `tests/` 配下に `encode_silent` / `decode_aac_silent` 相当のテストが (helpers 経由で) 移設される。
- `helpers.rs` に `encoder_config` が一元化され、`src/lib.rs::tests` 側の複製が消える。
- 既存の統合テストが引き続きパスする。

## 解決方法

1. `src/lib.rs:1039-1188` の `mod tests` を削除する。
2. `encode_silent` (L1071-1090) の内容を `tests/test_encoder.rs` に移設する。テスト名も `tests/` 側の命名規則 (`encoder_*` プレフィックス) に揃える。
3. `decode_aac_silent` (L1092-1134) の内容を `tests/test_decoder.rs` に移設する。既存の `decode_multiple_aac_packets_no_duplicate_feeds` と重複する部分は削るか統合する。
4. `init_encoder` の 3 分岐 (正常 128000 / 未指定 / 無効 1000) のうち、統合テスト側でカバーされていない「正常な 128000 での成功」を `tests/test_encoder.rs` に 1 件追加する (`encoder_new_accepts_valid_bitrate` 等)。
5. `init_decoder_mp3` / `init_decoder_opus` / `test_supported_codecs` の削除。
6. 併せて本 issue と同じスコープの issue 0020 (`.unwrap()` を `.expect()` に置き換える) を先または同時に対応することで、削除対象コードの規約違反を先に潰しておく。

なお、テストのログメッセージ言語混在 (issue 0019) やヘルパー配置 (issue 0035) は別 issue で扱う。本 issue は「重複解消」に限定する。
