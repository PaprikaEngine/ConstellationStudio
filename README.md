# Constellation Studio

**次世代リアルタイム映像処理プラットフォーム**

Rust + Ash Vulkan を核とした、ノードベースの映像処理システム。個人配信者から大手放送局まで、2D映像からVR/XR映像まで対応する革新的なメディア制作プラットフォームです。

## 🚀 Phase 1: ローカルスタンドアロン（2D基盤）

現在Phase 1の基盤構築が完了し、以下の機能が実装されています：

### ✅ 完了済み
- **Rustワークスペース**: 5つのコアクレートによるモジュラー設計
- **Vulkan基盤**: Ash + 高速メモリプール + GPU並列処理
- **ノードシステム**: Input/Output/Effect/Audio/Tallyの包括的ノード
- **React フロントエンド**: TypeScript + React Flow による直感的UI
- **型安全通信**: Serde + UUID による完全な型安全性

### 🔧 技術スタック

#### バックエンド
- **Rust**: メモリ安全 + 最高性能
- **Ash Vulkan**: 超低遅延GPU処理（<1.2ms@1080p目標）
- **マルチプラットフォーム**: Windows/macOS/Linux対応

#### フロントエンド  
- **React + TypeScript**: 型安全な開発体験
- **React Flow**: プロフェッショナルなノードエディタ
- **Vite**: 高速開発環境

## 📋 開発ロードマップ

現在の開発状況は[GitHubのIssue](https://github.com/PaprikaEngine/ConstellationStudio/issues)で管理されています：

### 🎯 Phase 1 残りタスク
1. **[#1 画面・ウィンドウキャプチャ実装](https://github.com/PaprikaEngine/ConstellationStudio/issues/1)** - 各プラットフォーム対応
2. **[#2 仮想Webカメラデバイス](https://github.com/PaprikaEngine/ConstellationStudio/issues/2)** - Zoom/Teams連携
3. **[#3 Vulkan最適化](https://github.com/PaprikaEngine/ConstellationStudio/issues/3)** - 性能目標達成
4. **[#4 フロントエンド連携](https://github.com/PaprikaEngine/ConstellationStudio/issues/4)** - 動作するアプリケーション
5. **[#5 基本エフェクト](https://github.com/PaprikaEngine/ConstellationStudio/issues/5)** - GPU最適化シェーダー
6. **[#6 TDD & CI/CD](https://github.com/PaprikaEngine/ConstellationStudio/issues/6)** - 品質保証

### 🔮 将来フェーズ
- **Phase 2**: プロ映像規格対応（SDI/NDI/SRT）
- **Phase 3**: クラウドスケーラブルシステム
- **Phase 4**: 3D/VR/XR対応

## 🏗️ プロジェクト構造

```
constellation-studio/
├── crates/
│   ├── constellation-core/      # コアエンジン (Ash Vulkan)
│   ├── constellation-vulkan/    # Vulkan処理・メモリ管理
│   ├── constellation-nodes/     # ノード実装
│   ├── constellation-pipeline/  # パイプライン管理
│   ├── constellation-audio/     # 音声処理
│   └── constellation-web/       # Web API (フロントエンド連携)
├── frontend/                    # React + TypeScript + React Flow
└── examples/                    # サンプル・ベンチマーク
```

## ⚡ 性能目標

| 解像度 | 目標レイテンシー | フレームレート |
|--------|------------------|----------------|
| 1080p | <1.2ms | 60fps+ |
| 4K | <6ms | 60fps |
| 8K | <24ms | 30fps |

## 🛠️ 開発環境

### 必要な環境
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (フロントエンド用)
# https://nodejs.org/ からインストール

# Vulkan SDK（開発・テスト用）
# https://vulkan.lunarg.com/ からインストール
```

### ビルドと実行
```bash
# バックエンドビルド
cargo build

# テスト実行
cargo test

# フロントエンド開発サーバー（準備中）
cd frontend && npm install && npm run dev
```

## 🤝 開発方針

- **TiDD**: テスト駆動開発で品質確保
- **適切な単位でコミット**: 機能単位での変更管理
- **GitHub Issue管理**: 透明性の高いタスク管理
- **段階的デリバリー**: Phase別の確実な進歩

## 📖 詳細仕様

プロジェクトの詳細な仕様・アーキテクチャについては [CLAUDE.md](./CLAUDE.md) をご参照ください。

## 🌟 革新的特徴

- **🔥 中間レンダリング共有**: 品質劣化のない高速処理
- **⚡ Ash Vulkan最適化**: C++同等性能 + Rust安全性
- **🎛️ ノードベースUI**: 直感的な映像処理パイプライン
- **📈 段階的スケーラビリティ**: 個人から放送局まで
- **🔒 メモリ安全**: Rustによる安全性 + 最高性能

---

**🤖 Generated with [Claude Code](https://claude.ai/code)**