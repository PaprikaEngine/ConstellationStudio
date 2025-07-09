#!/bin/bash

# CI と同じチェックをローカルで実行するスクリプト

set -e

echo "🔍 Running CI checks locally..."

# 1. フォーマットチェック
echo "📝 Checking formatting..."
cargo fmt --all -- --check
if [ $? -eq 0 ]; then
    echo "✅ Formatting check passed"
else
    echo "❌ Formatting check failed"
    echo "Run 'cargo fmt --all' to fix formatting issues"
    exit 1
fi

# 2. Clippy チェック（CIと同じ設定）
echo "🔍 Running clippy..."
cargo clippy --workspace --all-targets --all-features -- -D warnings -A clippy::too_many_arguments -A clippy::if_same_then_else -A clippy::items_after_test_module -A clippy::map_clone -A clippy::get_first -A dead_code -A unused_variables -A unexpected_cfgs -A clippy::uninlined_format_args
if [ $? -eq 0 ]; then
    echo "✅ Clippy check passed"
else
    echo "❌ Clippy check failed"
    exit 1
fi

# 3. ビルドチェック
echo "🔨 Building workspace..."
cargo build --workspace --verbose
if [ $? -eq 0 ]; then
    echo "✅ Build passed"
else
    echo "❌ Build failed"
    exit 1
fi

# 4. 単体テスト
echo "🧪 Running unit tests..."
cargo test --workspace --lib --verbose
if [ $? -eq 0 ]; then
    echo "✅ Unit tests passed"
else
    echo "❌ Unit tests failed"
    exit 1
fi

# 5. 統合テスト
echo "🔗 Running integration tests..."
CI=true cargo test --workspace --test '*' --verbose
if [ $? -eq 0 ]; then
    echo "✅ Integration tests passed"
else
    echo "❌ Integration tests failed"
    exit 1
fi

echo "🎉 All CI checks passed locally!"