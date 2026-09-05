# src/lib.rs::tests の .unwrap() を .expect("MESSAGE") に置き換える

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-replace-unwrap-with-expect
- Polished: 2026-07-31

## 目的

shiguredo-rust 規約「`.unwrap()` ではなく `.expect("MESSAGE")` を使用すること」に反し、`src/lib.rs::tests` に `.unwrap()` 呼び出しが残っている状態を解消する。

## 優先度根拠

Medium とする。テストコードとはいえ、規約は「lib / tests / examples 全て」を対象とし、失敗時の情報量を担保するために `.expect("MESSAGE")` を使う方針を明確にしている。テスト重複解消 (issue 0018) で `src/lib.rs::tests` を削除する場合は本 issue は自動的に消化される可能性があるが、独立した規約整合の観点で立てておく。

## 現状

`src/lib.rs:1167, 1177, 1185` に以下の `.unwrap()` が 3 箇所ある。

```rust
let aac_lc = codecs
    .iter()
    .find(|c| c.codec == AudioCodecType::AacLc)
    .unwrap();
...
let mp3 = codecs
    .iter()
    .find(|c| c.codec == AudioCodecType::Mp3)
    .unwrap();
...
let opus = codecs
    .iter()
    .find(|c| c.codec == AudioCodecType::Opus)
    .unwrap();
```

find が None を返した場合、`.unwrap()` は情報量のないパニックメッセージ (`called Option::unwrap() on a None value`) を出すのみで、どのコーデックのエントリが取れなかったかがログに残らない。

## 完了条件

- `src/lib.rs::tests` から `.unwrap()` が消える。
- 置換後の `.expect(...)` メッセージは日本語 (issue 0019 と整合)。
- issue 0018 で `src/lib.rs::tests` 全体を削除する場合は本 issue は不要になる旨をコメントで残す。

## 解決方法

以下のように置換する。

```rust
let aac_lc = codecs
    .iter()
    .find(|c| c.codec == AudioCodecType::AacLc)
    .expect("AAC-LC エントリが必ず存在するはず");
```

MP3 / Opus も同様に日本語メッセージで置き換える。

issue 0018 で `src/lib.rs::tests` を削除する場合は、本 issue は 0018 に統合されて消化される。作業順序としては 0018 を先に片付ければ本 issue はクローズできる。
