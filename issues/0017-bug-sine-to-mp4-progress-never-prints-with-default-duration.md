# examples/sine_to_mp4.rs の進捗表示が既定 5 秒では 1 度も出ない不具合を修正する

- Priority: High
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-sine-to-mp4-progress-condition
- Polished: 2026-07-31

## 目的

`examples/sine_to_mp4.rs` の 1 秒ごと進捗表示が、既定引数 (`--duration 5`) では 1 度も表示されず、README で紹介している `--duration 10` でも 8 秒時点に 1 回だけしか表示されない不具合を修正する。サンプルコードとしての第一印象を損ねる。

## 優先度根拠

High とする。サンプルコードは利用者が本クレートを触って感触を掴む最初の接点であり、「進捗が動かない」と誤解されると製品品質そのものへの信頼を落とす。修正コストが小さく、リグレッションリスクも低いため即修正できる。

## 現状の問題

`examples/sine_to_mp4.rs` の `main` 内、`sample_offset.is_multiple_of(SAMPLE_RATE as usize)` による進捗判定ブロック:

```rust
sample_offset += chunk_samples;
...
if sample_offset.is_multiple_of(SAMPLE_RATE as usize) {
    let sec = sample_offset / SAMPLE_RATE as usize;
    println!("  {sec}/{duration_secs:.0}s encoded");
}
```

`chunk_samples` は 1024 (`FRAMES_PER_PACKET`) 固定で加算されるため、`sample_offset` が 48000 (`SAMPLE_RATE`) の倍数になるのは `lcm(1024, 48000) = 384,000` サンプルごと (= 8 秒に 1 回) となる。

- `--duration 5` (既定): 進捗が 1 度も出ない
- `--duration 10` (README のコマンド例): 8 秒時点で 1 度だけ出る
- `--duration 8` 以上: 8 秒ごとに 1 回ずつ出る

サンプルコマンド `cargo run --example sine_to_mp4 -- --bitrate 256000 --duration 10 --freq 880 --output tone.mp4` では進捗表示が 8 秒時点の 1 回だけで、既定の `--duration 5` では全く出ないため、利用者は「フリーズしているのか正常に動いているのか判別できない」印象を受ける。

## 完了条件

- `--duration 1` 以上のいずれの値でも、秒が繰り上がるたびに進捗が表示される (表示は 48000 サンプル境界をまたいだ直後のチャンク処理時になるため、厳密な 1 秒時点からは最大 896 サンプル分遅れる)。小数秒の `--duration` では既存の表示形式により分母が丸められた表示になるが、本 issue の対象外とする。
- 具体的な確認として、`cargo run --example sine_to_mp4 -- --duration 5 --output /tmp/tone5.mp4` 実行時に `  1/5s encoded`〜`  5/5s encoded` の 5 行が出力される (最終行は最終チャンクが `total_samples` を超えて処理されることで表示される)。README のコマンド例 (`--duration 10`) でも `  1/10s encoded`〜`  10/10s encoded` の 10 行が出力される。
- 既存の出力 (`Encoding AAC ...` / `Done: ...`) はそのまま維持される。
- サンプルコード全体の変更は最小限に留める。
- `CHANGES.md` の develop に [FIX] として追記する。

## 解決方法

「秒が繰り上がったら出す」実装に変える。例:

```rust
let prev_sec = (sample_offset - chunk_samples) / SAMPLE_RATE as usize;
let cur_sec = sample_offset / SAMPLE_RATE as usize;
if cur_sec > prev_sec {
    println!("  {cur_sec}/{duration_secs:.0}s encoded");
}
```

コメントで「chunk_samples (1024) と SAMPLE_RATE (48000) の lcm が 384,000 サンプル (8 秒) のため、is_multiple_of では 1 秒ごとの表示にならない。秒の繰り上がり (cur_sec > prev_sec) で判定する」旨を残す。
