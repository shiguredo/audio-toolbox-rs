# build.rs で xcrun --show-sdk-path の失敗を検査するようにする

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-build-rs-check-xcrun-status
- Polished: 2026-09-04

## 目的

`build.rs` が `xcrun --show-sdk-path` の実行結果を検査せず、失敗しても空の SDK パスをそのまま以降で使ってしまう不具合を修正する。Xcode Command Line Tools 未導入時のエラー原因を明確に伝えるようにする。

## 優先度根拠

Medium とする。初回セットアップで Xcode CLT を入れ忘れた新規開発者は、以降の bindgen のバインディング生成失敗という意味不明なエラーで詰まる。build.rs は macOS 依存の入口であり、エラー時の一次診断メッセージの品質は重要。正常時には影響しないエラー診断品質の改善であり、機能・動作を壊すバグ (0014 / 0015 / 0017 相当) ではないため High にはしない。

## 現状

`build.rs` の `main` 内の `Command::new("xcrun")` による `--show-sdk-path` 呼び出し:

```rust
let output = Command::new("xcrun")
    .arg("--show-sdk-path")
    .output()
    .expect("failed to execute `xcrun` command");
let sdk_dir = PathBuf::from(
    String::from_utf8(output.stdout)
        .expect("invalid path")
        .trim(),
);
```

- `output.status.success()` を確認していない。
- 失敗時は `output.stdout` が空になり、`sdk_dir` が空パスとなる。以降の `sdk_dir.join(...)` は相対パスになり、symlink はダングリングのまま作成されるが、bindgen のバインディング生成 (`failed to generate bindings`) で clang の file not found エラーが出てビルドが失敗する。
- `output.stderr` の内容がエラーメッセージに含まれず、原因診断が困難。

### 再現手順

1. Xcode Command Line Tools が未導入の macOS 環境で `cargo build` を実行する (`xcrun` は非ゼロ終了し stderr に診断メッセージを出す)。
2. ビルドが「failed to generate bindings」という意味不明なエラーで失敗する。

## 完了条件

1. `xcrun` の非ゼロ終了を検出して明確なエラーで panic するようになる。
2. 非ゼロ終了時の panic メッセージに stderr の内容が含まれる。
3. 非ゼロ終了時の panic メッセージに「Is Xcode Command Line Tools installed?」のヒントが含まれる。
4. `xcrun` が成功したが空の SDK パスを返した場合も panic する (「`xcrun --show-sdk-path` returned empty path」)。
5. `xcrun` の起動自体に失敗した場合 (PATH に存在しない場合) も、spawn 失敗メッセージに「Is Xcode Command Line Tools installed?」のヒントが含まれる。
6. 既存の macOS ビルドが引き続き成功する (`cargo check --workspace` と `cargo test --workspace -- --test-threads=1` が成功する)。
7. `CHANGES.md` の develop 直下 (### misc ではなく) に [FIX] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `  - @ユーザー名` を含む) に従う。

## 検証方法

- 失敗経路 (非ゼロ終了): 事前に `cargo clean` を実行してビルドスクリプトの再実行を強制してから (build.rs の再実行条件に PATH は含まれないため)、一時的に PATH の先頭に非ゼロ終了し stderr に診断メッセージを出力する `xcrun` を模したシェルスクリプト (`#!/bin/sh` 付きで実行可能ビットを付けた `printf 'fake xcrun error' >&2; exit 1` 相当) を置いた状態で `cargo build` を実行し、panic メッセージに stderr の内容と「Is Xcode Command Line Tools installed?」が含まれることを確認する (手動検証のみでコードにテストは追加しない。AGENTS.md のモック・スタブ禁止に抵触しない)。
- 失敗経路 (空パス): 終了 0 で空出力を返す `xcrun` を模したシェルスクリプトで同様に検証し、「`xcrun --show-sdk-path` returned empty path」で panic することを確認する。
- 失敗経路 (spawn 失敗): 事前に `cargo clean` を実行してビルドスクリプトの再実行を強制してから、PATH から `xcrun` を外した状態で `cargo build` を実行し、expect メッセージに「Is Xcode Command Line Tools installed?」が含まれることを確認する。
- 成功経路: 通常の PATH で `cargo check --workspace` と `cargo test --workspace -- --test-threads=1` が成功することを確認する。

## 解決方法

`build.rs` の `main` 内の `Command::new("xcrun")` 呼び出し部分を以下のように書き換える。

```rust
let output = Command::new("xcrun")
    .arg("--show-sdk-path")
    .output()
    .expect("failed to execute `xcrun` command; Is Xcode Command Line Tools installed?");
if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let detail = if stderr.is_empty() {
        String::new()
    } else {
        format!(": {stderr}")
    };
    panic!(
        "`xcrun --show-sdk-path` failed (status={}){}\nIs Xcode Command Line Tools installed?",
        output.status,
        detail
    );
}
let sdk_dir = PathBuf::from(
    String::from_utf8(output.stdout)
        .expect("invalid path")
        .trim(),
);
if sdk_dir.as_os_str().is_empty() {
    panic!("`xcrun --show-sdk-path` returned empty path");
}
```

エラーメッセージは英語 (AGENTS.md「ログメッセージは全て英語にすること」)。

issue 0026 (build.rs の DOCS_RS スタブ拡張) と同じ build.rs を編集するため、並行作業時はコンフリクトに注意する (編集箇所は別のため直接の衝突は少ない)。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [FIX] build.rs で xcrun --show-sdk-path の失敗を検査して明確なエラーで panic するようにする` と担当者行 `  - @ユーザー名`)。
