# `AudioStreamBasicDescription` / `AudioBufferList` の `zeroed().assume_init()` を安全な初期化に置き換える

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

`MaybeUninit::<T>::zeroed().assume_init()` は、**型 `T` が全ビット 0 を合法状態として許す**場合に限り安全とされる。bindgen 生成型と **Apple ヘッダの実際の定義**の差により、**将来の SDK / bindgen 更新**でリスクが変わりうる。

**補足:** `AudioBufferList` の `assume_init` は **`Encoder::new` / `Decoder::new` だけでなく**、`encode_impl` / `decode_impl` でも **呼び出しのたびに**行われている。置き換え時は **全箇所**を同方針で扱う。

**フィールドを明示的に代入してから `assume_init`** するパターンや、**スコープ付き `MaybeUninit`** に寄せることで、レビュー可能性と堅牢性を上げたい。

## 受け入れ条件の目安

- `AudioStreamBasicDescription` と `AudioBufferList` で **初期化の置き方が異なる**場合は、型ごとに **安全な手順を設計メモまたはコメント**に残す。
- `src/lib.rs` および `src/codec_info.rs` の該当箇所を **同方針**で更新する。
- 振る舞いは **現状と同等**（リグレッションテストで確認）。

## 参考（該当コード）

- `src/lib.rs`: `Encoder::new`、`Decoder::new`、`encode_impl`、`decode_impl`
- `src/codec_info.rs`: `create_probe_converter`
