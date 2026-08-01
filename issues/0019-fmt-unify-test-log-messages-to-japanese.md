# テストのログメッセージを日本語に統一する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-unify-test-log-messages-to-japanese
- Polished: 2026-07-31

## 目的

AGENTS.md「テストのログメッセージは全て日本語にすること」に反し、`tests/*.rs` / `src/lib.rs::tests` の `expect` / `expect_err` / `assert!` / `assert_eq!` / `assert_ne!` / `panic!` メッセージが日本語と英語で混在している状態を解消し、日本語に統一する。

## 優先度根拠

Medium とする。テスト失敗時のログ出力の一貫性は重要だが、動作そのものには影響しない。AGENTS.md の必守項目に反しているため、規約整合として近いうちに直したい。

## 現状

日本語と英語が混在している主な箇所:

- `src/lib.rs::tests` の `encode_silent` / `decode_aac_silent` 内の `expect("create encoder error")`、`expect("encode error")`、`expect("finish error")`、`expect("decode error")` 等の英語 (issue 0018 で削除・移設される。0018 の解決方法で移設テストのメッセージは日本語で書くと明記済み)。
- `tests/test_encoder.rs` の `encoder_new_rejects_zero_sample_rate` / `encoder_new_rejects_zero_channels` の `assert!(msg.contains("Encoder::new(sample_rate)"))` 等 (これは `Error::function` の英語文字列を照合しているので日本語化すべきでない・許容)、`encoder_new_accepts_each_bitrate_control_mode` / `encoder_new_accepts_each_codec_quality` の assert 第 2 引数 (英語)、`encoder_next_frame_none_when_no_encoded_data` / `encoder_finish_on_empty_pcm_does_not_panic` の `expect("encoder")` / `expect("finish")` 等の英語。
- `tests/test_codec_info.rs` の `supported_codecs_has_expected_length_and_unique_codec_entries` の assert メッセージ (`"supported_codecs must not contain duplicate ..."`)、`supported_codecs_aac_lc_decode_and_encode` / `supported_codecs_mp3_decode_only_typically` / `supported_codecs_opus_decoding_supported` の `expect("AAC-LC entry")` 等の英語。
- `tests/test_decoder.rs` の `decoder_aac_stereo_48k` の `expect("Decoder::new must succeed on this platform")`、`decoder_new_rejects_zero_sample_rate` / `decoder_new_rejects_zero_channels` の `assert!(msg.contains("Decoder::new(input_sample_rate)"))` 等 (同上・許容)、`decode_empty_then_decode_non_empty_does_not_error` の `expect("empty decode")` / `expect("second decode after empty")` / `assert!("third decode without next_frame must fail")`、`decode_second_without_next_frame_returns_error` の `expect("first decode")` / `expect_err("second decode must fail")` / `assert!("unexpected error: {msg}")` (第 1 引数の `msg.contains("previous packet not consumed") && msg.contains("status=-50")` は例外・許容)、`decode_finish_then_next_frame_returns_none_without_input` の `expect("finish")` / `expect("next_frame")` 等の英語と、`finish_after_empty_decode_does_not_error` / `decode_then_loop_next_frame_consumes_all_output` の `expect("空の decode")` / `expect("finish 呼び出し")` 等の日本語が同一ファイル内で混在。
- `tests/include/helpers.rs` は日本語のみで対象外。

## 完了条件

- 全ての `expect` / `expect_err` / `assert!` / `assert_eq!` / `assert_ne!` / `panic!` メッセージが日本語になる (メッセージを持たない assert は対象外)。
- 唯一の例外は、`Error::function` の英語文字列や `Error` の Display 出力 (status 部を含む) の英語文字列を照合する assert の第 1 引数 (`assert!(msg.contains("Encoder::new(sample_rate)"))` 等)。これは実装側の英語識別子・フォーマットを照合しているのでそのまま。assert 系マクロの失敗時メッセージ引数 (assert_eq! / assert_ne! では第 3 引数) は日本語化する。
- 日本語化の確認は、`tests/` 配下と `src/lib.rs::tests` (0018 未実施で着手する場合のみ) を grep し、英語の `expect` / `expect_err` / `assert` (assert_eq! / assert_ne! を含む) / `panic` メッセージが残っていないことを確認する (上記の例外を除く)。
- issue 0018 を先に実施した場合は `src/lib.rs::tests` は削除済みのため、対象は `tests/` 配下のみで判定する。
- 既存のテストが引き続きパスする。
- `CHANGES.md` の develop / `### misc` に [UPDATE] として追記する (追記エントリに issue 番号・issue ファイル名は含めない。issue 0022 の管轄)。

## 解決方法

issue 0018 を先に実施する (0018 完了時点では `src/lib.rs::tests` は削除・移設済みで、移設テストは日本語メッセージで書かれるため、本 issue の対象は `tests/test_encoder.rs` / `tests/test_decoder.rs` / `tests/test_codec_info.rs` に限定される)。0018 未実施で本 issue に着手する場合は `src/lib.rs::tests` も対象に含める。

以下のファイルの英語メッセージを機械的に日本語に置換する。

- `src/lib.rs::tests` (0018 未実施の場合のみ)
- `tests/test_encoder.rs`
- `tests/test_codec_info.rs`
- `tests/test_decoder.rs` (混在している英語のみ)
- `tests/include/helpers.rs` (0018 移設時に英語メッセージが混入した場合のみ。現状は日本語のみで対象外)

0018 実施後に着手する場合は対象メッセージが削除・移設済みのため、残っている英語メッセージを grep で再特定する。

置換例:

- `expect("create encoder error")` → `expect("エンコーダー生成に失敗した")`
- `expect("encode error")` → `expect("encode に失敗した")`
- `expect("finish error")` → `expect("finish に失敗した")`
- `expect("finish")` → `expect("finish 呼び出し")`
- `expect("next_frame")` → `expect("next_frame 呼び出し")`
- `expect("decode error")` → 呼び出し元に応じて `expect("decode に失敗した")` / `expect("next_frame に失敗した")` を区別する (失敗箇所の判別を失わせないため)
- `expect("AAC-LC entry")` → `expect("AAC-LC エントリが必ず存在するはず")`
- `assert!(..., "Encoder::new with BitRateControlMode::{mode:?} failed: {:?}", r.err())` → `assert!(..., "Encoder::new が BitRateControlMode::{mode:?} で失敗した: {:?}", r.err())`

対象文字列そのものが英語である assert (`msg.contains(...)`) はそのままとする (詳細は完了条件の例外を参照)。

なお、`Error::function` の文字列自体を変更する issue 0034 を先に実施する場合は、照合 assert の対象文字列の更新は 0034 側に含まれるため本 issue の対象外とする。`supported_codecs_mp3_decode_only_typically` を改名・緩和する issue 0037 とは、どちらの順序でも互いの現状引用が古くなり得るが、対象メッセージの全数検証は本 issue の grep 手順で担保される。
