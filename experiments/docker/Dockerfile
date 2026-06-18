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
# Fully-qualified registry: Podman refuses unqualified short-names (Docker auto-prepends docker.io/).
ARG CUDA_IMAGE=docker.io/nvidia/cuda:12.8.1-devel-ubuntu24.04
FROM ${CUDA_IMAGE}

ARG LLAMA_CUDA=ON
# Blackwell (5080) is sm_120; the spread keeps the image usable on older cards too
# (Turing 75 → Ada 89 → Hopper 90 → Blackwell 120).
ARG CUDA_ARCHS="75;80;86;89;90;120"
ARG LLAMA_CPP_REF=master

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        git curl ca-certificates build-essential cmake ninja-build ccache \
        python3 python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Rust (for `myc-check`) — rustup stable is ≥ MSRV 1.92.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
# uv — the project's Python runner.
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.cargo/bin:/root/.local/bin:${PATH}"

# PREFLIGHT (seconds, before the ~10-min CUDA compile): prove this toolchain links an executable
# against a shared library that has UNDEFINED symbols — the exact shape of the llama.cpp link
# (llama-cli vs libggml-cuda.so's runtime-injected cu* symbols). If --allow-shlib-undefined is not
# honored this FAILS FAST here instead of after the whole compile. Its own cached layer.
RUN set -eux; cd /tmp; \
    printf 'int undefined_runtime_symbol(void);\nint lib_fn(void){return undefined_runtime_symbol();}\n' > pf_lib.c; \
    printf 'int lib_fn(void);\nint main(void){return lib_fn();}\n' > pf_main.c; \
    gcc -shared -fPIC pf_lib.c -o libpfprobe.so; \
    gcc pf_main.c -o pf_probe -L/tmp -lpfprobe -Wl,--allow-shlib-undefined; \
    echo "PREFLIGHT OK: exe links against a .so with undefined symbols (CUDA driver-link mechanism)"; \
    rm -f pf_lib.c pf_main.c libpfprobe.so pf_probe

# Build llama.cpp's server + cli with CUDA, put them on PATH so `--serve` finds them.
# --allow-shlib-undefined: libcuda.so.1 (the CUDA *driver*) is absent at build time and injected by
# the host at runtime (CDI). The exe link must tolerate those shlib-provided cu* symbols — this is
# exactly what upstream llama.cpp's .devops/cuda.Dockerfile does.
# ccache (cache mount): persists compiled objects across builds — re-runs (even after editing this
# recipe, which invalidates the layer) reuse the CUDA compile and only re-link, instead of a full
# ~10-min recompile. The cache survives a failed RUN, so a link error never costs the compile again.
RUN --mount=type=cache,target=/ccache,sharing=locked \
    git clone --depth 1 --branch "${LLAMA_CPP_REF}" https://github.com/ggml-org/llama.cpp /opt/llama.cpp \
    && CCACHE_DIR=/ccache CCACHE_MAXSIZE=8G cmake -S /opt/llama.cpp -B /opt/llama.cpp/build -G Ninja \
         -DCMAKE_BUILD_TYPE=Release \
         -DGGML_CUDA="${LLAMA_CUDA}" \
         -DCMAKE_CUDA_ARCHITECTURES="${CUDA_ARCHS}" \
         -DLLAMA_CURL=OFF \
         -DCMAKE_EXE_LINKER_FLAGS=-Wl,--allow-shlib-undefined \
         -DCMAKE_C_COMPILER_LAUNCHER=ccache \
         -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
         -DCMAKE_CUDA_COMPILER_LAUNCHER=ccache \
    && CCACHE_DIR=/ccache cmake --build /opt/llama.cpp/build --target llama-server llama-cli -j \
    && CCACHE_DIR=/ccache ccache --show-stats \
    && ln -s /opt/llama.cpp/build/bin/llama-server /usr/local/bin/llama-server \
    && ln -s /opt/llama.cpp/build/bin/llama-cli   /usr/local/bin/llama-cli \
    && rm -rf /opt/llama.cpp/.git

# Model cache lives on a named volume (see compose) so a fetched .gguf persists.
ENV MYCELIUM_LLM_MODEL_DIR=/models
WORKDIR /workspace/experiments
ENTRYPOINT ["bash"]
