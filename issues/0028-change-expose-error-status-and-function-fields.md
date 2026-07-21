# Error 型の status / function フィールドを公開して呼び出し側が OSStatus を握れるようにする

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/change-expose-error-fields
- Polished:

## 目的

`Error` 型の `status: i32` / `function: &'static str` フィールドは現状 private で、crate 外からアクセサ経由でも取得できない。README / SKILL.md はこれらを public として案内しているため実装とドキュメントが齟齬している。フィールドを `pub` にする (もしくはアクセサを追加する) ことで実装とドキュメントを整合させ、呼び出し側が OSStatus を握って分岐する典型ユースケースに対応する。

## 優先度根拠

Medium とする。動作影響は無いが、実装とドキュメントの齟齬は利用者にとって「書いてあるのに使えない」状態で信頼を損なう。呼び出し側 (Hisui 等) が特定 OSStatus をハンドリングする実装を組む際、現状は `Display` 文字列の正規表現剥がしという壊れやすい方式しか取れない。次回リリース (2026.2.0) の破壊的変更として `Error` を整えるチャンス。

## 現状

`src/lib.rs:29-45`:

```rust
#[derive(Debug)]
pub struct Error {
    /// AudioConverter API が返したステータスコード (OSStatus)
    status: i32,
    /// エラーが発生した API 関数名
    function: &'static str,
}
```

- フィールドに `pub` が無い。
- `impl Error` にはアクセサ (`pub fn status(&self) -> i32` 等) も無い。
- `Error::check` も `pub` ではないため crate 外から `Error` を組み立てる手段は無い (これは意図的で正しい)。

一方 README `### Error` (`README.md:219-224`) と SKILL.md (`skills/shiguredo-audio-toolbox/SKILL.md:73-77`) はテーブル記法で以下を列挙し、他型の public フィールドと区別が付かない書き方をしている。

```
| フィールド | 型 | 説明 |
| --- | --- | --- |
| `status` | `i32` | ... |
| `function` | `&'static str` | ... |
```

## 完了条件

- `Error::status` / `Error::function` を crate 外から取得できるようになる (`pub` 化またはアクセサ追加)。
- README / SKILL.md の記述と実装が整合する。
- 後方互換性 (2026.2.0 のマイナー / メジャー変更として妥当か) を判断して CHANGES.md に `[ADD]` / `[CHANGE]` として記載する。

## 解決方法

以下のいずれかで対応する。

### 案 A: フィールドを pub 化

```rust
#[derive(Debug)]
#[non_exhaustive]
pub struct Error {
    pub status: i32,
    pub function: &'static str,
}
```

`#[non_exhaustive]` を付ければ将来フィールド追加が破壊的変更にならない。ただし `pub` フィールド + `#[non_exhaustive]` は「読めるが構築はできない」意味論になる (crate 外から `Error { ... }` は書けなくなる)。

### 案 B: アクセサ追加

```rust
impl Error {
    /// AudioConverter API が返したステータスコード (OSStatus) を返す。
    pub fn status(&self) -> i32 {
        self.status
    }

    /// エラーが発生した API 関数名を返す。
    pub fn function(&self) -> &'static str {
        self.function
    }
}
```

案 A よりも柔軟 (将来内部表現を変えても API を維持できる) だが、フィールドアクセス構文が使えなくなり README / SKILL.md も「フィールド」ではなく「メソッド」として書き直す必要がある。

推奨は案 A + `#[non_exhaustive]`。案 A の場合、`Error::function` の命名一貫性問題 (issue 0034) と合わせて、`function` フィールドの意味論を再定義する余地もある。

対応時は CHANGES.md に `[ADD]` として明記する。

なお、issue 0034 (`Error::function` の命名一貫性) と密接に関係するため、両 issue を同じブランチで扱うか、順序を決めて別ブランチで扱うか検討する。
