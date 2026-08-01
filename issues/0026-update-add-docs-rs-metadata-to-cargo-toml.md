# Cargo.toml に [package.metadata.docs.rs] を追加して docs.rs で公開 API がドキュメント化されるようにする

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/update-docs-rs-metadata
- Polished: 2026-07-31

## 目的

`Cargo.toml` に `[package.metadata.docs.rs]` セクションを追加して、docs.rs 上で macOS ターゲット向けにビルドさせる。現状 `impl Encoder` / `impl Decoder` / `supported_codecs()` は `#[cfg(target_os = "macos")]` で gate されているため、docs.rs の既定 x86_64-unknown-linux-gnu ビルドではドキュメントから消えている。

## 優先度根拠

Medium とする。crate 利用者が docs.rs でドキュメントを確認しても README で紹介している API の doc コメント (利用方法の解説) が一切表示されない状態は、利用者体験を大きく損ねる。修正は Cargo.toml への追加と build.rs の DOCS_RS スタブ拡張で済む。

## 現状

- `src/lib.rs` の `impl Encoder` / `impl Decoder`、`src/codec_info.rs` の `supported_codecs()` は `#[cfg(target_os = "macos")]` で gate されている。
- docs.rs は既定で `x86_64-unknown-linux-gnu` でビルドするため、これらの `impl` / 関数は cfg が偽になりドキュメントに現れない。macOS gate されたメソッドへの intra-doc link も 15 箇所 unresolved になる。docs.rs のランディングページのプラットフォーム一覧も x86_64-unknown-linux-gnu のみで、macOS ターゲットのドキュメントは生成されていない。
- `build.rs` の DOCS_RS スタブ経路 (`std::env::var("DOCS_RS").is_ok()` 分岐) は `OpaqueAudioConverter` / `AudioConverterRef` / `AudioStreamPacketDescription` の 3 識別子のみの最小構成で、Linux ターゲットのビルドには十分だが、macOS ターゲットのビルドには不足する。なお CHANGES.md の 2026.1.0 記録 (`bindings_docs_stub.rs` を追加) はコミット 15c51e3 で build.rs にインライン化済みで、実ファイルは存在しない。

## 完了条件

1. `Cargo.toml` に以下が追加される (`grep -n "metadata.docs.rs" Cargo.toml` が 1 件、`grep -n 'default-target = "aarch64-apple-darwin"' Cargo.toml` が 1 件)。
   ```toml
   [package.metadata.docs.rs]
   default-target = "aarch64-apple-darwin"
   ```
2. `build.rs` の DOCS_RS スタブが macOS ターゲットでコンパイル可能な識別子 (型・定数・`extern "C"` 宣言) をカバーし、ローカルの `DOCS_RS=1 cargo doc --no-deps --target aarch64-apple-darwin` が成功して、生成された HTML (`target/aarch64-apple-darwin/doc/shiguredo_audio_toolbox/struct.Encoder.html` 等) に doc コメント本文 (例: 「エンコーダーインスタンスを生成する」) が含まれる (`grep` で確認)。
3. canary publish 後に docs.rs のビルドが成功し、バージョン直指定 URL (`https://docs.rs/crate/shiguredo_audio_toolbox/<バージョン>/builds` で成功を確認後) の `Encoder` 構造体ページ / `Decoder` 構造体ページ / `supported_codecs` 関数ページで doc コメントが表示される。ビルド後のプラットフォーム一覧を確認し、`targets = []` の要否判断結果を本 issue に記録する。docs.rs のビルドは publish 後にキューイングされ、完了までに時間がかかる場合がある。ビルド失敗時は原因調査と修正の後、canary.py でバージョンを上げ直して再 publish する。
4. doc_cfg 導入の別 issue (doc カテゴリ) が作成される。
5. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

1. `Cargo.toml` に上記 `[package.metadata.docs.rs]` セクションを追加する。`x86_64-apple-darwin` は動作要件外 (README は macOS (arm64) のみ) のため targets には追加しない。なお targets を未指定にすると docs.rs は既定ターゲット群 (linux / windows を含む) もビルドするとされるため、macOS 以外の不完全ドキュメントがプラットフォーム一覧に残る可能性がある。`targets = []` の併記による macOS 専用化は、canary publish でビルド後のプラットフォーム一覧を確認してから判断し、結果を本 issue に記録する。
2. `build.rs` の DOCS_RS スタブを、macOS ターゲットでコンパイルされる全識別子 (型・sys の参照で使用される全定数 (`kAudio*` に加えて `kMPEG4Object_AAC_LC` / `noErr` を含む)・`extern "C"` 関数宣言) をカバーするよう拡張する。
3. ローカルで `DOCS_RS=1 cargo doc --no-deps --target aarch64-apple-darwin` を実行し、生成された HTML に `Encoder::new` / `Decoder::new` / `supported_codecs` の doc コメントが含まれることを確認する。CI の docs-rs ジョブは Linux ホストターゲットのみの検証で macOS ターゲットを検証できないため、ローカル検証が必須 (CI への追加は行わない)。
4. canary.py でバージョンを上げ、タグを push して release.yml で publish し、完了条件 3 に従って docs.rs で確認する。
5. `CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: [UPDATE] docs.rs 向けに macOS ターゲットのドキュメントビルドを設定する)。

doc_cfg バッジ (`#[cfg_attr(docsrs, doc(cfg(...)))]`) の導入は、docs.rs 上でのターゲット別表示の改善が目的の別作業のため、本 issue では行わず別 issue (doc カテゴリ) として切り出す。

issue 0027 (include 変更) と同じ Cargo.toml を編集するため、並行作業時はコンフリクトに注意する。

docs.rs のビルド環境については <https://docs.rs/about/builds>、metadata 仕様は <https://docs.rs/about/metadata> を参照。
