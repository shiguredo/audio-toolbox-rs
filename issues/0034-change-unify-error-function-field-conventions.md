# Error::function フィールドの命名一貫性を確保する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/change-unify-error-function-field-conventions
- Polished: 2026-09-05

## 目的

`Error::function` に格納される文字列が「Rust 関数名 + 引数説明」「AudioToolbox API 関数名」「Rust 関数名 + プロパティ名」など複数のフォーマットで混在している状態を整理し、rustdoc の定義 (「エラーが発生した API 関数名」) と整合させる。あるいは rustdoc を実態に合わせて「エラーが発生した箇所を示す短い識別子」に緩める。

## 優先度根拠

Medium とする。動作影響は無いが、呼び出し側が `Display` 文字列や `Error::function` (issue 0028 で pub フィールドとして公開される) を使ってエラーハンドリングを組む際、書式の一貫性が無いと分岐が書けない。

## 現状

`Error::function` に入る値の分布:

- 引数バリデーション: `"Encoder::new(sample_rate)"`, `"Encoder::new(channels)"`, `"Decoder::new(input_sample_rate)"`, `"Decoder::new(input_channels)"`
- 事前条件チェック: `"Decoder::decode(previous packet not consumed)"`
- AudioToolbox API 関数名: `"AudioConverterNew"`, `"AudioConverterFillComplexBuffer"`
- AudioToolbox API + プロパティ名: `"AudioConverterSetProperty(BitRateControlMode)"`, `"AudioConverterSetProperty(EncodeBitRate)"`, `"AudioConverterSetProperty(CodecQuality)"`, `"AudioConverterSetProperty(SoundQualityForVBR)"`
- Rust 関数名 + フィールド名: `"Encoder::encode_impl(mDataByteSize)"`, `"Decoder::decode_impl(mDataByteSize)"`, `"Decoder::decode_impl(mDataByteSize alignment)"`

rustdoc (`src/lib.rs` の `Error::function` フィールドの doc comment) は「エラーが発生した API 関数名」と定義しているため、`Encoder::new(sample_rate)` などの Rust バリデーションエラーはこの説明に合致しない。同じ意味の説明が README.md の `### Error` 節 (「エラーが発生した API 関数名」) と `skills/shiguredo-audio-toolbox/SKILL.md` の「エラー型」節 (75 行目付近の「失敗した API 関数名」表記、77 行目付近の `Display` 書式の記述 `"[shiguredo_audio_toolbox] <function>() failed: status=<status>"`) にもある。SKILL.md は「エンコーダー側」節 (57 行目付近の `function: "Encoder::new(...)"`)・「デコーダー側」節 (67・69 行目付近の `function: "Decoder::new(...)"` / `function: "Decoder::decode(previous packet not consumed)"`) に旧書式の実例を含む。

## 完了条件

1. 案 B (書式統一 + rustdoc 緩和) で一貫性が確保される。案 A (enum 化) は issue 0028 の pub フィールド化と排他のため不採用。`src/lib.rs` の `function: "` / `Error::check(status, "` の全箇所 (現状: 14 種 16 箇所。うち `AudioConverterNew` / `AudioConverterFillComplexBuffer` は各 2 箇所) が解決方法の対応表の新書式に変換される (`rg -n 'function: "|Error::check\(status, "' src/lib.rs` の全結果が新書式に合致する)。箇所数は現状のものであり、本 issue より先に実装される他の issue で追加される箇所 (例: issue 0015 の null 検査追加による `"AudioConverterNew"` の 3 箇所目) も変換対象に含める。
2. issue 0028 の後に別ブランチで実施する (0028 で確定済み)。
3. テストコードの照合 assert (`tests/test_encoder.rs` の `encoder_new_rejects_zero_sample_rate` / `encoder_new_rejects_zero_channels`、`tests/test_decoder.rs` の `decoder_new_rejects_zero_sample_rate` / `decoder_new_rejects_zero_channels`、および issue 0028 で追加された `function ==` の等価 assert) が新書式に更新される (`decode_second_without_next_frame_returns_error` の照合文字列 `previous packet not consumed` は新書式でも成立するため変更不要)。
4. 既存の全テストが引き続きパスし、`cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --workspace -- -D warnings` が成功する。
5. README.md の `### Error` 節の `function` 説明、SKILL.md の各節 (「エンコーダー側」「デコーダー側」「エラー型」) の `function` 書式表記と `Display` 書式記述が新書式・新文言と整合する。
6. `CHANGES.md` の develop 直下 (### misc ではなく通常エントリ。種別順に従い先頭に挿入) に [CHANGE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

案 B (書式統一 + rustdoc 緩和) で対応する。案 A (enum 化) は issue 0028 の pub フィールド化と排他のため不採用。

1. `src/lib.rs` の `Error::function` フィールドの rustdoc を「エラーが発生した箇所を示す短い識別子」に緩める。README.md の `### Error` 節と SKILL.md の各節 (「エンコーダー側」「デコーダー側」「エラー型」) の `function` 説明・書式表記も合わせて更新する (SKILL.md の「エンコーダー側」節の `function: "Encoder::new(...)"` と「デコーダー側」節の `function: "Decoder::new(...)"` / `function: "Decoder::decode(previous packet not consumed)"` の実例も新書式に合わせる)。
2. `Error::function` に入る文字列を全て以下の対応表の新書式に統一する:

   | 現状 | 新書式 |
   |---|---|
   | `"Encoder::new(sample_rate)"` | `"Encoder::new/sample_rate"` |
   | `"Encoder::new(channels)"` | `"Encoder::new/channels"` |
   | `"Decoder::new(input_sample_rate)"` | `"Decoder::new/input_sample_rate"` |
   | `"Decoder::new(input_channels)"` | `"Decoder::new/input_channels"` |
   | `"Decoder::decode(previous packet not consumed)"` | `"Decoder::decode/previous packet not consumed"` |
   | `"AudioConverterNew"` | `"AudioConverterNew"` (不変) |
   | `"AudioConverterFillComplexBuffer"` | `"AudioConverterFillComplexBuffer"` (不変) |
   | `"AudioConverterSetProperty(BitRateControlMode)"` | `"AudioConverterSetProperty/BitRateControlMode"` |
   | `"AudioConverterSetProperty(EncodeBitRate)"` | `"AudioConverterSetProperty/EncodeBitRate"` |
   | `"AudioConverterSetProperty(CodecQuality)"` | `"AudioConverterSetProperty/CodecQuality"` |
   | `"AudioConverterSetProperty(SoundQualityForVBR)"` | `"AudioConverterSetProperty/SoundQualityForVBR"` |
   | `"Encoder::encode_impl(mDataByteSize)"` | `"Encoder::encode_impl/mDataByteSize"` |
   | `"Decoder::decode_impl(mDataByteSize)"` | `"Decoder::decode_impl/mDataByteSize"` |
   | `"Decoder::decode_impl(mDataByteSize alignment)"` | `"Decoder::decode_impl/mDataByteSize alignment"` |

   issue 0028 で追加される pub `function` フィールドは、この書式で値を格納する。
3. テストコードの照合 assert (完了条件 3 の一覧) を新書式に合わせて更新する。照合 assert の対象文字列の更新は本 issue 側に含まれ、issue 0019 の対象外となる。
4. `Display` 実装の書式を「`[{}] {} failed: status={}`」に変更する (`function` がスラッシュ区切りになると「`{}() failed`」の `()` が常に不自然になるため)。SKILL.md の `Display` 書式記述も合わせて更新する。
5. 最後に `cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --workspace -- -D warnings` で確認する。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [CHANGE] Error::function の書式を統一する`)。
