#!/bin/bash
# tokensbyte opensource
# (c) 2026 tokensbyte.ai
# @copyright      Copyright netbcloud/wstianxia 
# @license        MIT (https://www.tokensbyte.ai/)

set -e

if ! command -v docker >/dev/null 2>&1 || ! docker --version >/dev/null 2>&1; then
    if [ -d "/Applications/Docker.app/Contents/Resources/bin" ]; then
        export PATH="/Applications/Docker.app/Contents/Resources/bin:${PATH}"
    fi
fi

export PROJECT_NAME=${PROJECT_NAME:-$(basename "$PWD")}

echo "==== 重启 TokensByte 前后端开发服务 ===="

# 1. 杀死残留进程
echo "🧹 正在清理 3000, 5173, 3001, 5174 端口的残留进程..."
lsof -ti:3000 | xargs kill -9 2>/dev/null || true
lsof -ti:5173 | xargs kill -9 2>/dev/null || true
lsof -ti:3001 | xargs kill -9 2>/dev/null || true
lsof -ti:5174 | xargs kill -9 2>/dev/null || true
pkill -9 -f cargo 2>/dev/null || true
pkill -9 -f cargo-watch 2>/dev/null || true
pkill -9 -f rustc 2>/dev/null || true
pkill -9 -f tokensbyte-server 2>/dev/null || true
rm -f ~/.cargo/.package-cache.lock 2>/dev/null || true
rm -f "${HOME}/Library/Caches/tokensbyte-dev/target_pro/.cargo-lock" 2>/dev/null || true
rm -f "${HOME}/Library/Caches/tokensbyte-dev/target_os/.cargo-lock" 2>/dev/null || true

# 2. 确保 Docker 中的 Postgres 正常运行（主工程 5432，开源版 5434）
echo "🐳 检查 Docker 运行状态..."
if ! docker info > /dev/null 2>&1; then
    echo "🐳 Docker 未运行，尝试启动 Docker..."
    if [ -d "/Applications/OrbStack.app" ]; then
        open -a OrbStack
    elif [ -d "/Applications/Docker.app" ]; then
        open -a Docker
    else
        echo "❌ 无法自动找到 Docker 应用程序，请手动启动。"
        exit 1
    fi
    echo "⏳ 等待 Docker 启动 (可能需要几十秒)..."
    for i in $(seq 1 60); do
        if docker info > /dev/null 2>&1; then
            echo "✅ Docker 已成功启动"
            break
        fi
        if [ "$i" -eq 60 ]; then
            echo "❌ Docker 启动超时，请手动检查 Docker 状态。"
            exit 1
        fi
        sleep 2
    done
fi

echo "🐳 重启 Docker 中的 Postgres (主工程 + 开源版)..."
docker compose -f docker-compose.yml -f docker-compose.dev.yml down 2>/dev/null || true
docker network rm tokensbyte-network 2>/dev/null || true
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d postgres postgres-os

# 3. 等待数据库就绪
echo "⏳ 等待主数据库 (5432) 就绪..."
for i in $(seq 1 30); do
    if docker exec "${PROJECT_NAME}-postgres" pg_isready -U tokensapi &>/dev/null; then
        echo "✅ 主数据库已就绪"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "❌ 主数据库启动超时"
        exit 1
    fi
    sleep 1
done

echo "⏳ 等待开源版数据库 (5434) 就绪..."
for i in $(seq 1 30); do
    if docker exec "${PROJECT_NAME}-postgres-os" pg_isready -U tokensapi &>/dev/null; then
        echo "✅ 开源版数据库已就绪"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "❌ 开源版数据库启动超时"
        exit 1
    fi
    sleep 1
done

# 4. 导出通用环境变量
export PATH="/opt/homebrew/bin:${PATH}"
export RUST_LOG="${RUST_LOG:-info}"
if command -v sccache >/dev/null 2>&1; then
    export RUSTC_WRAPPER="sccache"
fi
if [ -x /opt/homebrew/bin/ld64.lld ]; then
    export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-fuse-ld=/opt/homebrew/bin/ld64.lld"
fi

# 5. 启动商业版 (主工程) 前后端 (后端 3000, 前端 5173)
echo "🚀 启动 [商业版] Rust 后端 (端口 3000) & Vite 前端 (端口 5173)..."
(
    cd backend
    export DATABASE_URL="postgres://tokensapi:tokensapi@localhost:5432/tokensapi"
    export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${HOME}/Library/Caches/tokensbyte-dev/target_pro}"
    nohup cargo watch -w src -x run > ../pro_backend_dev.log 2>&1 &
)
(
    cd frontend
    export VITE_API_TARGET="http://127.0.0.1:3000"
    nohup npm run dev -- --port 5173 > ../pro_frontend_dev.log 2>&1 &
)
# 软连接兼容主工程日志文件名
cp pro_backend_dev.log backend_daemon.log 2>/dev/null || true
cp pro_frontend_dev.log frontend_daemon.log 2>/dev/null || true

sleep 1

# 6. 启动开源版 前后端 (后端 3001, 前端 5174)
if [ -d "opensource/backend" ] && [ -d "opensource/frontend" ]; then
    if [ ! -d "opensource/frontend/node_modules" ]; then
        echo "📦 正在安装开源版前端依赖..."
        (cd opensource/frontend && npm install)
    fi
    echo "🚀 启动 [开源版] Rust 后端 (端口 3001) & Vite 前端 (端口 5174)..."
    (
        cd opensource/backend
        export DATABASE_URL="postgres://tokensapi:tokensapi@127.0.0.1:5434/tokensapi"
        export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${HOME}/Library/Caches/tokensbyte-dev/target_os}"
        export DATA_DIR="data_opensource"
        export PORT="3001"
        nohup cargo watch -w src -x run > ../../os_backend_dev.log 2>&1 &
    )
    (
        cd opensource/frontend
        export VITE_API_TARGET="http://127.0.0.1:3001"
        nohup npm run dev -- --port 5174 > ../../os_frontend_dev.log 2>&1 &
    )
fi

echo ""
echo "🎉 商业版与开源版服务重启指令已发送，后台全速运行中："
echo "   🛍  [商业版/主工程] 前端: http://localhost:5173  | 后端: http://localhost:3000"
echo "   📖  [开源版]         前端: http://localhost:5174  | 后端: http://localhost:3001"
echo ""
echo "- 商业版日志: pro_backend_dev.log / pro_frontend_dev.log"
echo "- 开源版日志: os_backend_dev.log / os_frontend_dev.log"

