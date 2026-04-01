# エンコーダー・コールバック内の整数乗算・キャストを checked にする

Created: 2026-04-01
Model: Claude Opus 4.5

**スコープ:** 本ブランチのミッションは **パニック・セグメンテーションフォルト（および FFI による未定義動作）** の防止に限定する。

## なぜこの対応が必要か

コールバック内で `packets as usize * channels`、`packets * channels as u32 * size_of::<i16>()`、`(this.pcm_buf.len() / channels) as u32` 等を用いている。**乗算のオーバーフロー**や **`usize` から `u32` への切り詰め**により、**条件判定や `drain` 範囲が壊れ**、**パニック**（例: `drain` の範囲不正）や **不正なメモリ操作**につながりうる。

**補足:** `Decoder::callback` の `io_number_data_packets` の **null 検査**は `issues/0006` と合わせて扱う。

## 受け入れ条件の目安

- オーバーフローしうる演算は **`checked_mul` / `try_from`** 等に置き換え、失敗時は **安全側**（エラーコード返却または `K_NO_MORE_INPUT` 等、設計に合わせる）。
- 32bit ターゲットでも **論理が破綻しない**こと。

## 参考（該当コード）

- `src/lib.rs`: `Encoder::callback`（`packets`、`channels`、`drain` 範囲）、`Decoder::callback`（`io_number_data_packets` の逆参照）
