# `Makefile` の `pbt` / `fuzz` ターゲットとリポジトリ構成を一致させる

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

ルート `Makefile` は `cargo test -p pbt` および `cargo fuzz list` を前提としているが、**単一クレートの `Cargo.toml` のみ**のワークスペースでは **`pbt` パッケージが存在しない**可能性がある。また **`fuzz/` ディレクトリ**が無い場合、`cargo fuzz` 系ターゲットは **失敗**する。

利用者・CI が **ドキュメント通りにコマンドを実行できない**状態を解消する必要がある。

## 受け入れ条件の目安

- **いずれか**: ワークスペースに `pbt` クレートと fuzz ターゲットを追加する、**または** `Makefile` と README を **現状の構成に合わせて修正**する。
- `make pbt` / `make fuzzing-list` が **意図どおり動く**こと（環境要件は README に明記）。

## 参考

- `Makefile`
- ルート `Cargo.toml`
- `tests/test_decoder.rs` 内の cargo-fuzz に関するコメント（整合性を取る）
