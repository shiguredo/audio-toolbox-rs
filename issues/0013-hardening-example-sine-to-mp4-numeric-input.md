# サンプル `sine_to_mp4` の CLI 数値を検証し不正値の黙受けを防ぐ

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

`examples/sine_to_mp4.rs` で `--freq` 等を `f64` としてパースし、正弦波生成で **`f64 as i16` キャスト**を行っている。

### 調査結果（Rust 言語仕様）

[Rust Reference（Type cast expressions / Numeric cast / 浮動小数点から整数へ）](https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast) では、浮動小数点から整数への `as` について次が規定されている。

- `NaN` は **0** になる。
- 整数の最大値を超える値（`INFINITY` を含む）は **その整数型の最大値に飽和**する。
- 整数の最小値未満（`NEG_INFINITY` を含む）は **最小値に飽和**する。

**ローカル検証（rustc 1.94.1）:** `f64::NAN as i16 == 0`、`f64::INFINITY as i16 == 32767`、`f64::NEG_INFINITY as i16 == -32768`、`1e100_f64 as i16 == 32767`。クレートの `rust-version`（`Cargo.toml`）でも **Reference に従う**前提でよいが、必要なら **MSRV で再確認**する。


したがって **`f64 as i16` は未定義動作ではなく**、危険度の主因は **UB ではなく**、**NaN や極端な値が黙って 0 や飽和値になり、無音やクリップした正弦になる**こと、および **`duration_secs` が極端に大きい**ときのループ・メモリ負荷である。サンプルは **お手本**として、**有限・意図した範囲の入力**を検証するのがよい。

## 受け入れ条件の目安

- `freq` / `duration_secs`（および必要なら `bitrate`）について **有限・合理的範囲**を満たさない場合は **エラーメッセージ**で終了する。
- `NonZeroU32::new(SAMPLE_RATE).unwrap()` は定数なので実質到達しないが、**サンプルとして `expect` を避ける**かコメントで意図を明示する。

## 参考（該当コード）

- `examples/sine_to_mp4.rs`: `generate_sine_pcm`、`main` の引数パース

## 参考（外部）

- Rust Reference（数値キャスト・浮動小数点 → 整数）: <https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast>
