# build.rs で xcrun --show-sdk-path の失敗を検査するようにする

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-build-rs-check-xcrun-status
- Polished:

## 目的

`build.rs` が `xcrun --show-sdk-path` の実行結果を検査せず、失敗しても空の SDK パスをそのまま以降で使ってしまう不具合を修正する。Xcode Command Line Tools 未導入時のエラー原因を明確に伝えるようにする。

## 優先度根拠

Medium とする。初回セットアップで Xcode CLT を入れ忘れた新規開発者は、以降の symlink 作成失敗という意味不明なエラーで詰まる。build.rs は macOS 依存の入口であり、エラー時の一次診断メッセージの品質は重要。

## 現状

`build.rs:41-49`:

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
- 失敗時は `output.stdout` が空になり、`sdk_dir` が空パスとなる。以降の `sdk_dir.join(...)` は空文字結合となり、symlink 作成 (`build.rs:62-63`) で意味不明なエラーが出る。
- `output.stderr` の内容がエラーメッセージに含まれず、原因診断が困難。

Xcode CLT 未導入時の `xcrun` は「xcode-select: error: no developer tools found」的なメッセージを stderr に出すため、これを含めた panic メッセージが望ましい。

## 完了条件

- `xcrun` の非ゼロ終了を検出して明確なエラーで panic するようになる。
- panic メッセージに stderr の内容が含まれる。
- 「Xcode Command Line Tools がインストールされていない可能性がある」旨のヒントが含まれる。
- 既存の macOS ビルドが引き続き成功する。

## 解決方法

`build.rs:41-49` を以下のように書き換える。

```rust
let output = Command::new("xcrun")
    .arg("--show-sdk-path")
    .output()
    .expect("failed to execute `xcrun` command; is Xcode Command Line Tools installed?");
if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    panic!(
        "`xcrun --show-sdk-path` failed (status={:?}): {}\nIs Xcode Command Line Tools installed?",
        output.status.code(),
        stderr.trim()
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
