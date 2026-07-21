# examples/sine_to_mp4.rs の進捗表示が既定 5 秒では 1 度も出ない不具合を修正する

- Priority: High
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-sine-to-mp4-progress-condition
- Polished:

## 目的

`examples/sine_to_mp4.rs` の 1 秒ごと進捗表示が、既定引数 (`--duration 5`) や README で紹介している `--duration 10` では 1 度も表示されない不具合を修正する。サンプルコードとしての第一印象を損ねる。

## 優先度根拠

High とする。サンプルコードは利用者が本クレートを触って感触を掴む最初の接点であり、「進捗が動かない」と誤解されると製品品質そのものへの信頼を落とす。修正コストが小さく、リグレッションリスクも低いため即修正できる。

## 現状の問題

`examples/sine_to_mp4.rs:173-176`:

```rust
sample_offset += chunk_samples; // += 1024 (FRAMES_PER_PACKET)
...
if sample_offset.is_multiple_of(SAMPLE_RATE as usize) {
    let sec = sample_offset / SAMPLE_RATE as usize;
    println!("  {sec}/{duration_secs:.0}s encoded");
}
```

`chunk_samples` は 1024 (`FRAMES_PER_PACKET`) 固定で加算されるため、`sample_offset` が 48000 (`SAMPLE_RATE`) の倍数になるのは `lcm(1024, 48000) = 6,144,000` サンプルごと (= 128 秒に 1 回) となる。

- `--duration 5` (既定): 進捗が 1 度も出ない
- `--duration 10` (README のコマンド例): 進捗が 1 度も出ない
- `--duration 128` 以上: 128 秒ごとに 1 度だけ出る

サンプルコマンド `cargo run --example sine_to_mp4 -- --bitrate 256000 --duration 10 --freq 880 --output tone.mp4` で進捗表示が全く出ないため、利用者は「フリーズしているのか正常に動いているのか判別できない」印象を受ける。

## 完了条件

- `--duration 1` 以上のいずれの値でも、秒が繰り上がるたびに進捗が表示される。
- 既存の出力 (`Encoding AAC ...` / `Done: ...`) はそのまま維持される。
- サンプルコード全体の変更は最小限に留める。

## 解決方法

「秒が繰り上がったら出す」実装に変える。例:

```rust
let prev_sec = sample_offset.saturating_sub(chunk_samples) / SAMPLE_RATE as usize;
let cur_sec = sample_offset / SAMPLE_RATE as usize;
if cur_sec > prev_sec {
    println!("  {cur_sec}/{duration_secs:.0}s encoded");
}
```

コメントで「chunk_samples が SAMPLE_RATE を割り切らないため、is_multiple_of ではなく秒の繰り上がりで判定する」旨を残す。
