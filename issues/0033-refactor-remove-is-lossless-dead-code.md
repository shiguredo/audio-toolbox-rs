# AudioCodecType::is_lossless() のデッドコードを削除する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-remove-is-lossless-dead-code
- Polished:

## 目的

`AudioCodecType::is_lossless()` (`src/codec_info.rs:88-90`) は現状の probe 対象 (AAC-LC / MP3 / Opus) では永久に false になるデッドコードのため削除する。呼び出し箇所の分岐もリテラル化する。

## 優先度根拠

Medium とする。動作は正しいが、読者に「無効な分岐」と誤解させ、将来 FLAC / ALAC を追加する際にも本来的な意味を再検討する必要がある。「Don't live with broken windows」の観点から早めに整理する。

## 現状

`src/codec_info.rs:88-90` の定義:

```rust
fn is_lossless(self) -> bool {
    matches!(self, Self::Flac | Self::Alac)
}
```

`src/codec_info.rs:275` の唯一の呼び出し:

```rust
output_format.mBitsPerChannel = if codec.is_lossless() { 16 } else { 0 };
```

`AudioCodecType::codec_types_for_supported_codecs()` (`src/codec_info.rs:39-41`) が返すのは `AacLc`, `Mp3`, `Opus` の 3 種のみ。`create_probe_converter` はこの 3 種に対してしか呼ばれないので、`is_lossless()` は常に false を返す。

## 完了条件

- `AudioCodecType::is_lossless()` が削除される。
- `create_probe_converter` の分岐がリテラル `0` に置き換わり、コメントで「現状の probe 対象は非ロスレスコーデックのみ」の旨を残す。
- 将来 FLAC / ALAC を probe に含める際は、is_lossless の再導入もしくは probe 対象ごとに `mBitsPerChannel` を指定する設計を検討する旨を issues/ で残す。

## 解決方法

1. `src/codec_info.rs:88-90` の `is_lossless` を削除する。
2. `src/codec_info.rs:275` を `output_format.mBitsPerChannel = 0;` に置き換え、コメントで理由を明記する。
3. 併せて `AudioCodecType` の未使用バリアント (`AacHe` / `AacHeV2` / `AacLd` / `AacEld` / `Flac` / `Alac`) の扱いを別 issue で再確認する (issue 0003 で「残す」判断があるが、is_lossless 削除と合わせて再検討する余地がある)。
