# docs.rs で macOS 専用 API に doc_cfg バッジを表示する

- Created: 2026-09-05
- Completed: {YYYY-MM-DD}
- Branch: feature/update-add-doc-cfg-badges
- Polished: 2026-09-05

## 目的

docs.rs のページ上で、macOS 専用 API（`impl Encoder` / `impl Decoder` / `supported_codecs()`）を利用しようとしている読み手に「この API は macOS (arm64) でのみ使える」ことを一目で分かるようにする。現状は doc コメントまたは README の動作要件を読まないとターゲット制約を把握できない。

## 優先度根拠

Medium とする。動作に影響はないが、Linux / Windows の利用者が macOS 専用 API を無駄にコンパイルしようとする誤解を防ぐ。issue 0026 で docs.rs の macOS ターゲットビルドが導入されるまでは docs.rs 上での確認ができないが、実装自体は先行して進められる独立した作業である。

## 現状

- `src/lib.rs` の `impl Encoder` / `impl Decoder` と `src/codec_info.rs` の `supported_codecs()` は `#[cfg(target_os = "macos")]` で gate されている。
- docs.rs のビルドは `docsrs` cfg を付与して rustdoc を実行するため、`#[cfg_attr(docsrs, doc(cfg(...)))]` を付ければターゲットバッジが表示されるが、現状どの公開 API にも付与されていない。
- README.md の動作要件には「macOS (arm64)」のみと明記されているが、rustdoc 上の各 API にはターゲット情報が表示されない。

## 完了条件

- `src/lib.rs` の冒頭に crate 属性 `#![cfg_attr(docsrs, feature(doc_cfg))]` が追加される。
- macOS gate された公開 API（`src/lib.rs` の `impl Encoder` / `impl Decoder` と `src/codec_info.rs` の `supported_codecs()`）の rustdoc に `#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]` が付与される。
- ローカルの nightly ツールチェーンで `RUSTDOCFLAGS="--cfg docsrs" cargo doc --no-deps` を実行し、対象 API のページにターゲットバッジが生成される。macOS 以外のホストでは issue 0026 の `DOCS_RS=1` + `--target aarch64-apple-darwin` 付きローカル検証経路を利用する。
- issue 0026 の完了後、docs.rs の macOS ターゲットページで `Encoder` / `Decoder` / `supported_codecs` にターゲットバッジが表示される。

## 解決方法

- `src/lib.rs` の冒頭に crate 属性として `#![cfg_attr(docsrs, feature(doc_cfg))]` を追加する。`doc(cfg(...))` は unstable な `doc_cfg` 機能であり、これを有効にしないと docs.rs（nightly）でもローカルの `--cfg docsrs` 付き doc でも `#[doc(cfg)]` がコンパイルエラー（E0658）になる。`docsrs` cfg 自体は docs.rs が自動で付与するため、cfg の有効化のための Cargo.toml の変更は不要（<https://docs.rs/about/builds> の「Detecting Docs.rs」を参照）。通常のビルド / doc（`docsrs` 未設定）では `cfg_attr` が無効になるため stable ツールチェーンには影響しない（CI の docs-rs ジョブは `DOCS_RS=1 cargo doc --no-deps` のため影響なし）。
- `src/lib.rs` の macOS gate された `impl Encoder` / `impl Decoder` と、`src/codec_info.rs` の `supported_codecs()` の doc 属性に `#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]` を追加する。`Encoder` / `Decoder` 構造体や `EncoderConfig` / `DecoderConfig` / `EncoderCodec` / `DecoderCodec` / `AudioCodecType` / `AudioCodecInfo` など cfg gate されていない公開アイテムは対象外（rustdoc に表示される macOS gate 済み公開 API は `impl Encoder` / `impl Decoder` のメソッド群と `supported_codecs()` のみ。`impl AudioCodecType` / `impl DecoderCodec` も gate されているがメソッドは private のため表示対象外。`doc_cfg` の有効化で cfg gate された公開アイテムには自動でバッジも表示されるが、対象を明示するため `doc(cfg(...))` を付与する）。
- 検証は docs.rs と同じ nightly ツールチェーンで行う（`rust-toolchain.toml` は stable のため `cargo +nightly doc --no-deps`。stable では `#![feature(doc_cfg)]` を有効にできないため、`RUSTDOCFLAGS="--cfg docsrs"` 付きの stable での検証は不能）。macOS 以外のホストでは issue 0026 の `DOCS_RS=1` + `--target aarch64-apple-darwin` 付きローカル検証経路を利用する。実際の docs.rs 上での表示確認は issue 0026 の完了後に行う（ローカルで生成した HTML でもバッジの有無は確認できる）。
