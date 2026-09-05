# docs.rs で macOS 専用 API に doc_cfg バッジを表示する

- Created: 2026-09-05
- Completed: {YYYY-MM-DD}
- Branch: feature/update-add-doc-cfg-badges
- Polished: {YYYY-MM-DD}

## 目的

docs.rs のページ上で、macOS 専用 API（`impl Encoder` / `impl Decoder` / `supported_codecs()`）を利用しようとしている読み手に「この API は macOS (arm64) でのみ使える」ことを一目で分かるようにする。現状は doc コメントまたは README の動作要件を読まないとターゲット制約を把握できない。

## 優先度根拠

Medium とする。動作に影響はないが、Linux / Windows の利用者が macOS 専用 API を無駄にコンパイルしようとする誤解を防ぐ。issue 0026 で docs.rs の macOS ターゲットビルドが導入されるまでは docs.rs 上での確認ができないが、実装自体は先行して進められる独立した作業である。

## 現状

- `src/lib.rs` の `impl Encoder` / `impl Decoder` と `src/codec_info.rs` の `supported_codecs()` は `#[cfg(target_os = "macos")]` で gate されている。
- docs.rs のビルドは `docsrs` cfg を付与して rustdoc を実行するため、`#[cfg_attr(docsrs, doc(cfg(...)))]` を付ければターゲットバッジが表示されるが、現状どの公開 API にも付与されていない。
- README.md の動作要件には「macOS (arm64)」のみと明記されているが、rustdoc 上の各 API にはターゲット情報が表示されない。

## 完了条件

- macOS gate された公開 API の rustdoc に `#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]` が付与される。
- ローカルで `RUSTDOCFLAGS="--cfg docsrs" cargo doc --no-deps` を実行し、対象 API のページにターゲットバッジが生成される。
- issue 0026 の完了後、docs.rs の macOS ターゲットページで `Encoder` / `Decoder` / `supported_codecs` にターゲットバッジが表示される。

## 解決方法

- `src/lib.rs` の `pub struct Encoder` / `pub struct Decoder` とその impl ブロック、`src/codec_info.rs` の `supported_codecs()` など、macOS gate された公開アイテムの doc 属性に `#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]` を追加する。
- docs.rs は自動で `docsrs` cfg を有効にするため Cargo.toml の変更は不要（<https://docs.rs/about/builds> の「Detecting Docs.rs」を参照）。
- 検証はローカルの `RUSTDOCFLAGS="--cfg docsrs"` 付きで行い、実際の docs.rs 上での表示確認は issue 0026 の完了後に行う（ローカルで生成した HTML でもバッジの有無は確認できる）。
