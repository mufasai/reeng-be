#!/bin/bash

echo "🧪 Testing local build..."
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust."
    exit 1
fi

echo "✅ Cargo found"
echo ""

# Try to build
echo "🔨 Building project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "Binary location: target/release/reengineering-tool-be"
    echo ""
    echo "To test locally, set environment variables and run:"
    echo "  export SURREAL_URL=localhost:8001"
    echo "  export SURREAL_NAMESPACE=yerico"
    echo "  export SURREAL_DATABASE=project_budget"
    echo "  export SURREAL_USERNAME=root"
    echo "  export SURREAL_PASSWORD=root"
    echo "  export JWT_SECRET=your-secret"
    echo "  ./target/release/reengineering-tool-be"
else
    echo ""
    echo "❌ Build failed!"
    exit 1
fi
