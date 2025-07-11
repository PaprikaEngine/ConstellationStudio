# Contributing to Constellation Studio

## Code Quality Requirements

**重要**: コミット前に必ず以下を実行してください：

### 1. コードフォーマット
```bash
cargo fmt --all
```

### 2. Lintチェック
```bash
cargo clippy --workspace --all-targets --all-features
```
**全てのClippyワーニングを解決してからコミットする**

### 3. テスト実行
```bash
cargo test --workspace --lib
```

### 4. CI準備スクリプト
ローカルでCIと同じチェックを実行：
```bash
./scripts/ci-check.sh
```

## 開発方針

### TiDD (Ticket-driven Development)
- GitHubのIssueでタスクを管理
- 適切な単位でコミットを作成
- 機能単位でブランチを分ける

### ブランチ戦略
- `main`: 本番リリース用
- `feature/issue-XX-description`: 機能開発用
- `fix/issue-XX-description`: バグ修正用

### コミットメッセージ
```
feat: 機能追加の説明

- 変更内容1
- 変更内容2

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

## アーキテクチャ

プロジェクト構造の詳細は[ARCHITECTURE.md](./ARCHITECTURE.md)を参照してください。

## Phase別開発

### Phase 1: 基本実装
- 画面・ウィンドウキャプチャ
- 仮想Webカメラ出力
- 基本ノードシステム

### Phase 2: プロフェッショナル対応
- SDI/NDI/SRT対応
- 高度フレーム制御
- プロ向け機能

### Phase 3: クラウド対応
- マイクロサービス化
- スケーラブルシステム
- グローバル配信

### Phase 4: 3D/VR/XR対応
- 3Dシーン処理
- VRデバイス統合
- XRレンダリング