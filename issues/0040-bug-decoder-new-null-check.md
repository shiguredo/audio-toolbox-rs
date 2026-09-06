# Decoder::new に Encoder::new と同様の null 検査を追加する

- Created: 2026-09-06
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-decoder-new-null-check
- Polished: {YYYY-MM-DD}

## 目的

`AudioConverterNew` 成功後の null 検査の方針を `Encoder::new` と `Decoder::new` で統一する。`Encoder::new` と `src/codec_info.rs` の `create_probe_converter` は null を検査するのに対し、`Decoder::new` だけが検査せずに保持するため、将来 null が渡った場合の挙動が箇所によって異なる。

## 現状

`src/lib.rs` の `Decoder::new` は `AudioConverterNew` の成功直後に null 検査を行わず、そのまま `Self` に保持する。null に対する `AudioConverterDispose` の挙動は Apple ドキュメントに明記されていないため、null を保持した `Decoder` の `Drop` や以降の `AudioConverterFillComplexBuffer` 呼び出しに null が渡る恐れがある。

一方、`src/lib.rs` の `Encoder::new` は成功後に `converter.is_null()` を検査し、null 時は dispose せずに `Error` を返す。`src/codec_info.rs` の `create_probe_converter` も同一の判定で null を除外する。

## 設計方針

`Decoder::new` に `Encoder::new` と同じ null 検査を追加する。null 時は dispose せずに `Error` を返して早期 return する。専用のガード型は導入しない。

## 完了条件

- `Decoder::new` が `AudioConverterNew` 成功後の null を検査し、null 時に `Error` を返す。
- 既存のテストが引き続きパスする。
- `CHANGES.md` の develop に [FIX] として追記する。

## 解決方法

1. `src/lib.rs` の `Decoder::new` で `AudioConverterNew` 成功後に `converter.is_null()` を検査し、null 時は `Error` を返して早期 return する (`Encoder::new` と同じ null 検査。null のため dispose は不要)。
2. 追加テストは不要。null 成功経路は Apple 側の実装依存で再現できない防御的経路であり、既存の `Decoder::new` 成功パスと引数バリデーションのテストが引き続きパスすることで回帰を検証できる。
