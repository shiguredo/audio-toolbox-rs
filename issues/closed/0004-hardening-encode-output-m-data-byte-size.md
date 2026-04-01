# エンコード経路で `mDataByteSize` を検証し `encoded_data[..size]` のパニックを防ぎ、異常値を明示的に扱う

Created: 2026-04-01
Completed: 2026-04-01
Model: Claude Opus 4.5

**スコープ:** 本ブランチのミッションは **パニック・セグメンテーションフォルト（および FFI による未定義動作）** の防止に限定する。

## なぜこの対応が必要か

**問題設定（二層を混同しない）:** (1) **Rust 側** — `mDataByteSize` を **`encoded_data[..size]` に使う前に**検証すれば防げるのは、**スライスによる境界外アクセス（パニック）**に限る。(2) **C 側・FFI** — フレームワークが **`mData` に渡した 4096 バイトを超えて実際に書き込んだ**場合、その越境は **`AudioConverterFillComplexBuffer` 呼び出しの内部で既に発生しうる**。呼び出し**後**に `mDataByteSize` を読んでも **その事象を未然には防げない**（検出・異常扱い・以降の Rust 側誤用の防止に寄与するにすぎない）。

`Encoder::encode_impl` は `AudioConverterFillComplexBuffer` 呼び出し**後**に `output_buffer_list.mBuffers[0].mDataByteSize` をそのまま `size` として `encoded_data[..size]` に用いている。`encoded_data` は **`ENCODE_BUF_SIZE`（4096）バイトのスタック配列**であり、`size` がこれを超えると **スライス境界でパニック**する。対応としては、**スライス前に `size` を検証し**、異常なら **`Result::Err` 等**とし、上記 **問題設定 (1)(2)** に沿って **Rust 側パニック**と **報告値の異常検知**を満たす。

## 受け入れ条件の目安

- `mDataByteSize` が `ENCODE_BUF_SIZE` を超える場合、**`Result::Err`** 等で呼び出し側に伝える（**クランプだけで成功扱いにしない**。**Rust 側**のパニック防止と異常値の明示が目的。**C 側の実越境を事後読み取りだけで防ぐ**ことを保証するものではない）。
- 正常系では **`encoded_data[..size]` が常に境界内**になること。
- 可能なら **macOS 上**で境界に関する **回帰テスト**または **エラーパス**の単体テストを追加する。

## ミッション適合性の確認

- **適合する。** 根拠: **`encoded_data[..size]`** は `size > ENCODE_BUF_SIZE` で **パニック**するため、**事前／事後のどちらであれ `size` を検証してからスライスすれば** Rust 側のパニックは防げる。C 側の実越境は **FFI 呼び出しの内部**で起きうるが、**検証は少なくとも Rust 側のパニック経路を閉じ、異常値を明示的に扱う**ことに効く。
- **注意:** **C 側越境の完全な防止**は、**単一事後読み取りだけでは保証できない**（上記「なぜ」のとおり）。ミッション上は **パニック防止と異常の明示**を主目的とする。

## 既存 issue との関係

- デコード出力側は `issues/0005-hardening-decode-output-m-data-byte-size.md`。

## 参考（該当コード）

- `src/lib.rs`: `encode_impl`、`ENCODE_BUF_SIZE`

## 解決方法

- `Encoder::encode_impl` で `AudioConverterFillComplexBuffer` 成功後、`mDataByteSize` を `usize` にした値が `ENCODE_BUF_SIZE` を超える場合は `kAudio_ParamError` 相当の `Error` を返し、`encoded_data[..size]` のパニックを防ぐようにした。
