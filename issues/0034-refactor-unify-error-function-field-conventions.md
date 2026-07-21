# Error::function フィールドの命名一貫性を確保する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-unify-error-function-conventions
- Polished:

## 目的

`Error::function` に格納される文字列が「Rust 関数名 + 引数説明」「AudioToolbox API 関数名」「Rust 関数名 + プロパティ名」など複数のフォーマットで混在している状態を整理し、rustdoc の定義 (「エラーが発生した API 関数名」) と整合させる。あるいは rustdoc を実態に合わせて「エラーが発生した箇所を示す短い識別子」に緩める。

## 優先度根拠

Medium とする。動作影響は無いが、呼び出し側が `Display` 文字列や `Error::function` (issue 0028 でアクセサ化される想定) を使ってエラーハンドリングを組む際、書式の一貫性が無いと分岐が書けない。issue 0028 と一体で扱うのが自然。

## 現状

`Error::function` に入る値の分布:

- 引数バリデーション: `"Encoder::new(sample_rate)"`, `"Encoder::new(channels)"`, `"Decoder::new(input_sample_rate)"`, `"Decoder::new(input_channels)"`, `"Decoder::decode(previous packet not consumed)"`
- AudioToolbox API 関数名: `"AudioConverterNew"`, `"AudioConverterFillComplexBuffer"`
- AudioToolbox API + プロパティ名: `"AudioConverterSetProperty(BitRateControlMode)"`, `"AudioConverterSetProperty(EncodeBitRate)"`, `"AudioConverterSetProperty(CodecQuality)"`, `"AudioConverterSetProperty(SoundQualityForVBR)"`
- Rust 関数名 + フィールド名: `"Encoder::encode_impl(mDataByteSize)"`, `"Decoder::decode_impl(mDataByteSize)"`, `"Decoder::decode_impl(mDataByteSize alignment)"`

rustdoc (`src/lib.rs:33-34`) は「エラーが発生した API 関数名」と定義しているため、`Encoder::new(sample_rate)` などの Rust バリデーションエラーはこの説明に合致しない。

## 完了条件

- 以下のいずれかで一貫性が確保される。
  - 案 A: `Error` を enum 化し、`ValidationError { context: &'static str }` / `AudioToolboxError { function: &'static str, property: Option<&'static str> }` のようにバリアントで分ける。
  - 案 B: `function` フィールドは維持しつつ rustdoc を「エラーが発生した箇所を示す短い識別子 (Rust 関数名 or Audio Toolbox API 名)」に緩め、命名は「`Encoder::new/sample_rate`」「`AudioConverterSetProperty/BitRateControlMode`」のようにスラッシュ区切りで統一する。
- 変更は issue 0028 (Error フィールド公開) と同一ブランチで扱うか、順序を決めて別ブランチで扱うか判断する。

## 解決方法

案 A (enum 化) が理想だが後方互換に対する影響が大きい。案 B (書式統一 + rustdoc 緩和) を推奨。

案 B の場合:

1. `src/lib.rs:33-34` の rustdoc を「エラーが発生した箇所を示す短い識別子」に緩める。
2. `Error::function` に入る文字列を全て見直し、以下のいずれかの書式に統一する:
   - 引数バリデーション: `"Encoder::new/sample_rate"`
   - API 呼び出し: `"AudioConverterNew"` (プロパティ無し)
   - プロパティ設定: `"AudioConverterSetProperty/BitRateControlMode"`
3. テストコードの `contains` assertion (`tests/test_encoder.rs:23,39,44` 等) を新書式に合わせて更新する。
4. issue 0028 で `pub fn function()` を出す場合は、この書式で公開する。

CHANGES.md には `[CHANGE]` として明記する (Display 文字列と `Error::function` の書式変更は破壊的変更相当)。
