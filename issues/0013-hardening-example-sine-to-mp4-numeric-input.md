# サンプル `sine_to_mp4` の CLI 数値入力を検証し未定義動作を防ぐ

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

`examples/sine_to_mp4.rs` で `--freq` 等を `f64` としてパースし、正弦波生成で **`as i16` キャスト**を行っている。**NaN / ±Inf** のとき **`f64 as i16` は Rust で未定義動作**となる。

また **`total_samples`** の算出やループ境界は、**極端に大きい `duration_secs`** で **意図しない挙動**（長時間ループ・メモリ圧迫）を招きうる。サンプルは **お手本**として、入力の **検証または拒否**を行うのがよい。

## 受け入れ条件の目安

- `freq` / `duration_secs`（および必要なら `bitrate`）について **有限・合理的範囲**を満たさない場合は **エラーメッセージ**で終了する。
- `NonZeroU32::new(SAMPLE_RATE).unwrap()` は定数なので実質到達しないが、**サンプルとして `expect` を避ける**かコメントで意図を明示する。

## 参考（該当コード）

- `examples/sine_to_mp4.rs`: `generate_sine_pcm`、`main` の引数パース
