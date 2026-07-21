# Cargo.toml に [package.metadata.docs.rs] を追加して docs.rs で公開 API がドキュメント化されるようにする

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/change-add-docs-rs-metadata
- Polished:

## 目的

`Cargo.toml` に `[package.metadata.docs.rs]` セクションを追加して、docs.rs 上で macOS ターゲット向けにビルドさせる。現状 `impl Encoder` / `impl Decoder` / `supported_codecs()` は `#[cfg(target_os = "macos")]` で gate されているため、docs.rs の既定 x86_64-unknown-linux-gnu ビルドではドキュメントから消えている。

## 優先度根拠

Medium とする。crate 利用者が docs.rs でドキュメントを確認しても README で紹介している API の実装解説が一切出ない状態は、利用者体験を大きく損ねる。修正は Cargo.toml への数行追加で済む。

## 現状

`src/lib.rs:263` `#[cfg(target_os = "macos")] impl Encoder`、`src/lib.rs:750` `#[cfg(target_os = "macos")] impl Decoder`、`src/codec_info.rs:123` `#[cfg(target_os = "macos")] pub fn supported_codecs()` は全て macOS gate されている。

docs.rs は既定で `x86_64-unknown-linux-gnu` でビルドするため、これらの `impl` / 関数は cfg が偽になりドキュメントに現れない。DOCS_RS スタブ経路 (`build.rs:19-35`) を通してもコンパイルはできるが `impl` は消えたまま。

## 完了条件

- `Cargo.toml` に以下相当のセクションが追加される。
  ```toml
  [package.metadata.docs.rs]
  default-target = "aarch64-apple-darwin"
  targets = ["aarch64-apple-darwin", "x86_64-apple-darwin"]
  ```
- 追加後、docs.rs 上で `Encoder::new` / `Decoder::new` / `supported_codecs` 等の実装解説が正しく表示される (canary publish で確認)。
- 併せて `#![cfg_attr(docsrs, feature(doc_cfg))]` と `#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]` の導入も検討する (nightly が必要になるため要判断)。

## 解決方法

1. `Cargo.toml` に上記 `[package.metadata.docs.rs]` セクションを追加する。
2. canary publish して docs.rs のビルドが通り、`impl` ブロックがドキュメントに現れることを確認する。
3. `#[doc(cfg)]` を採用するかは別途検討する。採用する場合は stable でビルドできるように feature gate を工夫する (`#[cfg_attr(docsrs, feature(doc_auto_cfg))]` 等)。

docs.rs のビルド環境については <https://docs.rs/about/builds> を参照。
