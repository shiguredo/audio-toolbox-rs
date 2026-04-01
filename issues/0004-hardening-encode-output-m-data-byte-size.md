# エンコード経路で `mDataByteSize` を検証しスタックバッファ越境と `encoded_data[..size]` のパニックを防ぐ

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

`Encoder::encode_impl` は `AudioConverterFillComplexBuffer` 呼び出し後に `output_buffer_list.mBuffers[0].mDataByteSize` をそのまま `size` として `encoded_data[..size]` に用いている。`encoded_data` は **`ENCODE_BUF_SIZE`（4096）バイトのスタック配列**であり、`size` がこれを超えると **スライス境界でパニック**する。

さらに、フレームワークが **4096 バイトを超えて `mData` に書き込む**場合、**C 側のバッファオーバーフロー**となり **未定義動作・セグフォ**の原因になる。Rust 側で **`mDataByteSize` が渡したバッファ容量を超えない**ことを検証し、異常時は **エラーとして返す**（パニックにしない）必要がある。

## 受け入れ条件の目安

- `mDataByteSize` が `ENCODE_BUF_SIZE` を超える場合、**`Result::Err`** 等で呼び出し側に伝える。**クランプして短いスライスだけ成功扱いにしない**（データ欠落・誤ったパケットを黙って返さないこと）。
- 正常系では **`encoded_data[..size]` が常に境界内**になること。
- 可能なら **macOS 上**で境界に関する **回帰テスト**または **エラーパス**の単体テストを追加する。

## 参考（該当コード）

- `src/lib.rs`: `encode_impl`、`ENCODE_BUF_SIZE`
