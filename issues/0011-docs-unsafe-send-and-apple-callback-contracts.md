# `unsafe impl Send` と AudioConverter コールバックに関する契約をドキュメント化する

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

1. **`unsafe impl Send for Encoder/Decoder`** は、**別スレッドへ所有権を移動**できることを意味する。`AudioConverterRef` が **スレッドセーフでない**場合、**誤用でデータ競合（UB）**になりうる。`Sync` を実装しない理由と合わせ、**利用上の禁止事項**を crate または型の rustdoc に書く必要がある。

2. コールバックで **`encoded_buf.as_mut_ptr()`** を渡している。**コールバック終了後に Apple がポインタを保持しない**ことは、安全性の重要な前提である。公式ドキュメントで裏付けられる範囲で **「同期・同一スレッド・非保持」**を明記する。

## 既存の記述との関係

`src/lib.rs` には既に **`Sync` を実装しない理由**や **AudioConverter がスレッドセーフでない**旨、`Encoder` / `Decoder` の **コールバックが同一スレッドで同期的**である旨の **コメント**がある。

本 issue で足すのは **既存コメントの rustdoc への昇格**、**`Send` による「別スレッドへ移動した後の誤用」**（例: 複数スレッドからの間接利用）の **明示的禁止**、および **コールバック内ポインタの寿命**を **利用者向けに読める形**にまとめたもの。

## 受け入れ条件の目安

- `Encoder` / `Decoder` またはクレートルートの **`#![doc = ...]` / 各 struct の `///`** に、**スレッド境界**と **コールバック内ポインタの寿命**を記載する（**上記と重複しないよう整理**する）。
- 参照可能なら **Apple Developer Documentation** の URL を rustdoc に含める（リンク切れに注意し、可能なら安定した節名で）。

## 参考（該当コード）

- `src/lib.rs`: `unsafe impl Send`、`Decoder::callback`（`encoded_buf` の `mData`）、既存の `Sync` / スレッド関連コメント
