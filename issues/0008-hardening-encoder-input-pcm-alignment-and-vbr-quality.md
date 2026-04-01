# エンコーダー入力の PCM 整合性・`vbr_quality`・デコーダー入力長の検証

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

1. **`pcm.len() % channels != 0`** のとき、`pcm_buf.len() / self.channels` 等の **切り捨て**により、`EncodedFrame.samples` や内部消費量の解釈が **意図とずれる**可能性がある。呼び出し契約を **エラーで強制**するか、**明文化した上で受け入れる**かを決める必要がある。

2. **`vbr_quality`** は rustdoc で **0〜127** と説明されているが、`Encoder::new` は **範囲外を拒否していない**。無効値は **AudioConverter がエラー**にする可能性はあるが、**早期に `Err`** した方が利用者に分かりやすい。

3. **`Decoder` の `encoded_buf.len() as u32`**（パケット記述・`mDataByteSize` 設定）では、**`u32::MAX` 超**で **切り詰め**が起こりうる（極端ケース）。方針として **`Err`** にするか検討する。**長さの実行時検証**は `issues/0005` と重なりうるため、**エンコーダー側の `as u32` とデコーダー側を同じ方針**で扱う。

## スコープ

- 本 issue は **PCM 整列・`vbr_quality`・`len` の `u32` キャスト**をまとめている。実装時に **issue を分割**してもよい。

## 受け入れ条件の目安

- PCM フレーム境界: **`Err`** または **明確な仕様**のいずれか（未指定なら **拒否**を推奨）。
- `vbr_quality`: **指定時のみ 0〜127** を満たさなければ **`Err`**（または `Option` の意味を rustdoc で固定）。
- 長さ `as u32`:** 方針決定後、必要なら **`Err`**（デコーダーは **0005** と整合）。

## 参考（該当コード）

- `src/lib.rs`: `Encoder::encode`、`Encoder::new`、`Decoder::decode` / `Decoder::callback`（`encoded_buf.len() as u32`）
