# テストのログメッセージを日本語に統一する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-unify-test-log-messages-to-japanese
- Polished:

## 目的

AGENTS.md「テストのログメッセージは全て日本語にすること」に反し、`tests/*.rs` / `src/lib.rs::tests` の `expect` / `assert!` / `panic!` メッセージが日本語と英語で混在している状態を解消し、日本語に統一する。

## 優先度根拠

Medium とする。テスト失敗時のログ出力の一貫性は重要だが、動作そのものには影響しない。AGENTS.md の必守項目に反しているため、規約整合として近いうちに直したい。

## 現状

日本語と英語が混在している主な箇所:

- `src/lib.rs:1073, 1079, 1084, 1096, 1102, 1108, 1112, 1120, 1121, 1125, 1126`
  - `expect("create encoder error")`, `expect("encode error")`, `expect("finish error")`, `expect("decode error")` 等の英語
- `tests/test_encoder.rs:23, 39, 44, 49, 71-73, 97-99, 105, 111, 112`
  - `assert!(msg.contains("Encoder::new(sample_rate)"))` (これは `Error::function` の英語文字列を照合しているので日本語化不能・許容)
  - `assert!(r.is_ok(), "Encoder::new with BitRateControlMode::{mode:?} failed: {:?}", r.err())` (assert メッセージ側は日本語化対象)
  - `expect("encoder")`, `expect("finish")` 等の英語
- `tests/test_codec_info.rs:15, 26, 38, 49`
  - `expect("AAC-LC entry")`, `expect("MP3 entry")`, `expect("Opus entry")`, `"supported_codecs must not contain duplicate ..."` 等の英語
- `tests/test_decoder.rs:11, 25, 39, 68-75, 81-87, 93-95, 153-156, 171, 174`
  - `expect("Decoder::new must succeed on this platform")`, `expect("empty decode")`, `expect("first decode")`, `expect("finish")` 等の英語と `expect("空の decode")`, `expect("finish 呼び出し")`, `expect("最初のパケットの decode")` 等の日本語が同一ファイル内で混在

## 完了条件

- 全ての `expect` / `assert!` / `panic!` メッセージが日本語になる。
- 唯一の例外は、`Error::function` の英語文字列を pattern match する assert (`msg.contains("Encoder::new(sample_rate)")` 等)。これは実装側の英語識別子を照合しているのでそのまま。
- 既存のテストが引き続きパスする。

## 解決方法

以下のファイルの英語メッセージを機械的に日本語に置換する。

- `src/lib.rs::tests` (issue 0018 で削除される場合はそちらの作業に含める)
- `tests/test_encoder.rs`
- `tests/test_codec_info.rs`
- `tests/test_decoder.rs` (混在している英語のみ)
- `tests/include/helpers.rs` (英語メッセージが残っていれば)

置換例:

- `expect("create encoder error")` → `expect("エンコーダー生成に失敗した")`
- `expect("encode error")` → `expect("encode 呼び出しに失敗した")`
- `expect("finish error")` → `expect("finish 呼び出しに失敗した")`
- `expect("decode error")` → `expect("decode / next_frame 呼び出しに失敗した")`
- `expect("AAC-LC entry")` → `expect("AAC-LC エントリが必ず存在するはず")`
- `assert!(..., "Encoder::new with BitRateControlMode::{mode:?} failed: {:?}", r.err())` → `assert!(..., "Encoder::new が BitRateControlMode::{mode:?} で失敗した: {:?}", r.err())`

「Encoder::new(sample_rate) を含む文字列を assert」のような、対象文字列そのものが英語である assert (`msg.contains("Encoder::new(sample_rate)")`) はそのままとする。ただし assert の第 2 引数 (失敗時メッセージ) は日本語化する。

なお、issue 0018 と近い作業なので同時期に対応するのが効率的だが、目的が異なる (issue 0018 は重複解消、本 issue は言語統一) ため別 issue として管理する。
