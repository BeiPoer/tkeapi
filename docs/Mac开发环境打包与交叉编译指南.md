# Mac 开发环境 Docker 镜像极速打包与交叉编译指南

> 本指南用于解决 Apple Silicon (M1/M2/M3/M4) Mac 开发机打包 **x86_64 (amd64) Linux 云服务器** Docker 镜像时构建缓慢、QEMU 模拟器崩溃及 Docker `cannot allocate memory` 爆内存问题。

---

## 💡 核心难题与解决思路

### 为什么在 Mac 上打包 x86_64 镜像会爆内存/崩溃？
- **QEMU 模拟器瓶颈**：Mac (ARM64) 通过 Docker 默认的 QEMU 模拟器去运行 x86_64 的 `rustc` 编译器，CPU 模拟效率极低。
- **内存暴顶**：Rust `cargo build --release` 在代码优化与二进制链接阶段并发内存消耗极大，经常触发 Docker 虚拟机的内存限制 (`SIGKILL / cannot allocate memory`)。

### 🚀 终极解决思路：宿主机原生交叉编译 + USE_PREBUILT 秒级打包
1. **彻底摆脱 QEMU**：使用 Mac 宿主机上的 **`cargo-zigbuild`** + **`zig`**，直接利用 Mac M 芯片的原生算力，交叉编译出标准 **Linux x86_64 ELF** 二进制文件（`tokensbyte-server-bin`）。
2. **秒级 Docker 打包 (`USE_PREBUILT=1`)**：将编译好的二进制文件直接 `COPY` 到 Docker 容器，跳过容器内 150+ 个 Rust 依赖库的全量编译。
   - ⚡️ **编译时间**：从 5 分钟缩短到 **30 秒**
   - 📦 **Docker 镜像生成**：**3 秒完成**
   - 🛡️ **稳定性**：**0 内存爆满，0 QEMU 崩溃**

---

## 🛠️ 环境准备（只需配置一次）

在 Mac 上打开终端，运行以下命令安装交叉编译环境：

```bash
# 1. 安装 zig 编译器
brew install zig

# 2. 安装 Rust 交叉编译工具链
rustup target add x86_64-unknown-linux-gnu
# 若打 arm64 Linux：rustup target add aarch64-unknown-linux-gnu
cargo install cargo-zigbuild
```

导出脚本会自动 `cargo zigbuild --features cross_compile`（vendored OpenSSL，解决 Mac 上找不到 Linux OpenSSL 的问题）。

---

## 📦 本地打包与离线镜像导出

`export-images.sh` / `push-images.sh`：Mac 选定目标架构后 **必须** 先 `cargo zigbuild` 生成匹配的 `tokensbyte-server-bin`（校验 ELF/架构），再以 `USE_PREBUILT=1` 打镜像；失败直接退出，不再掉进 Docker 内 cargo（避免 OOM）。已有正确 bin 则复用；`FORCE_ZIGBUILD=1` 重编；仅调试用 `ALLOW_DOCKER_CARGO=1`。

### 1. 导出 Docker 镜像
在项目根目录运行：
```bash
./export-images.sh
```
- Apple Silicon 选 `1`（`linux/amd64`）即可走 zigbuild；工具未装齐时会提示安装或回退 arm64 / CI。
- 导出目录：`docker-images/*.tar`。

---

## 🚀 云服务器部署导入

将打包生成的 `.tar` 离线文件上传到您的 x86_64 Linux 云服务器后，在服务器上运行：

```bash
# 1. 导入后端与前端镜像
docker load -i tokensbyte-os-backend-x86_64.tar
docker load -i tokensbyte-os-frontend-x86_64.tar

# 2. 使用导入的本地镜像启动（不拉取、不在服务器构建）
BACKEND_IMAGE=tokensbyte-os-backend:latest \
FRONTEND_IMAGE=tokensbyte-os-frontend:latest \
docker compose up -d --no-build --pull never
```
