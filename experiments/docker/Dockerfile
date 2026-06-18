# KC-2 experiment runner — CUDA + Python(uv) + Rust + llama.cpp(server, CUDA).
#
# One image that runs the WHOLE pipeline with `--serve`: it has llama-server (GPU),
# the Rust toolchain to build `myc-check` (the Mycelium checker), and uv to run the
# experiment. The repo is BIND-MOUNTED at runtime (not COPYed), so reports/logs land
# on the host and the image stays generic.
#
# Best-effort (honesty): the CUDA build below is NOT exercisable in this design-phase
# sandbox (no GPU here). Verify on the host with `… run --rm kc2 nvidia-smi` before a
# long run. CPU-only fallback: build with `--build-arg LLAMA_CUDA=OFF`.

# syntax=docker/dockerfile:1
# CUDA 12.8+ is required for Blackwell (RTX 5080 = sm_120).
ARG CUDA_IMAGE=nvidia/cuda:12.8.1-devel-ubuntu24.04
FROM ${CUDA_IMAGE}

ARG LLAMA_CUDA=ON
# Blackwell (5080) is sm_120; the spread keeps the image usable on older cards too
# (Turing 75 → Ada 89 → Hopper 90 → Blackwell 120).
ARG CUDA_ARCHS="75;80;86;89;90;120"
ARG LLAMA_CPP_REF=master

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        git curl ca-certificates build-essential cmake ninja-build \
        python3 python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Rust (for `myc-check`) — rustup stable is ≥ MSRV 1.92.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
# uv — the project's Python runner.
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.cargo/bin:/root/.local/bin:${PATH}"

# Build llama.cpp's server + cli with CUDA, put them on PATH so `--serve` finds them.
RUN git clone --depth 1 --branch "${LLAMA_CPP_REF}" https://github.com/ggml-org/llama.cpp /opt/llama.cpp \
    && cmake -S /opt/llama.cpp -B /opt/llama.cpp/build -G Ninja \
         -DCMAKE_BUILD_TYPE=Release \
         -DGGML_CUDA="${LLAMA_CUDA}" \
         -DCMAKE_CUDA_ARCHITECTURES="${CUDA_ARCHS}" \
         -DLLAMA_CURL=OFF \
    && cmake --build /opt/llama.cpp/build --target llama-server llama-cli -j \
    && ln -s /opt/llama.cpp/build/bin/llama-server /usr/local/bin/llama-server \
    && ln -s /opt/llama.cpp/build/bin/llama-cli   /usr/local/bin/llama-cli \
    && rm -rf /opt/llama.cpp/.git

# Model cache lives on a named volume (see compose) so a fetched .gguf persists.
ENV MYCELIUM_LLM_MODEL_DIR=/models
WORKDIR /workspace/experiments
ENTRYPOINT ["bash"]
