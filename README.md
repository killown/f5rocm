# MATRIX VOICE STUDIO: F5-TTS ORCHESTRATOR

Low-latency desktop wrapper for F5-TTS synthesis. Optimized for AMD RDNA 4 (9060 XT) via ROCm. Manages VRAM state transitions between LLM (Ollama) and Diffusion Transformer (F5-TTS) workloads.

## SYSTEM ARCHITECTURE

The application functions as a native Rust orchestrator using the following execution flow:

1. RESOURCE ARBITRATION: Executes 'ollama stop' to clear VRAM.
2. CONTAINER LIFECYCLE: Triggers 'docker compose up' in the engine directory.
3. INFERENCE INJECTION: Executes 'docker exec' to start the Gradio server (infer_gradio.py) bound to 0.0.0.0.
4. ASYNC HEALTH CHECK: Polls localhost:7860 using Tokio/Reqwest until 200 OK is received.
5. VIEWPORT TRANSITION: Navigates the Tauri WebView to the local engine URL and reveals the window.

## CORE DEPENDENCIES

- Runtime: Tokio 1.38 (Full)
- Frontend: Tauri 2.0 (Wry/Webkit2GTK)
- Backend: Docker Engine + Docker Compose
- Hardware: AMD ROCm 6.0+ (HSA_OVERRIDE_GFX_VERSION=11.0.0)
- Communication: Reqwest (Async)

## INSTALLATION AND SETUP

### 1. DIRECTORY STRUCTURE

The orchestrator resolves paths dynamically. Ensure the F5-TTS engine directory (containing src/ and docker-compose.yml) is accessible. The Rust logic defaults to a configurable relative or absolute path defined in the logic constants.

### 2. DOCKER CONFIGURATION

The Dockerfile must utilize a ROCm-compatible PyTorch base. The docker-compose.yml must map /dev/dri and /dev/kfd to allow the container direct access to the RDNA 4 hardware.

### 3. COMPILATION

A placeholder frontend directory is required for the Tauri macro to resolve context during the build phase.

$mkdir -p dist && touch dist/index.html$ cargo tauri build

## EXECUTION

To initialize the development environment:

$ cargo tauri dev

The main thread remains non-blocking. The UI remains hidden during the "Cold Start" phase (Model loading into VRAM), typically lasting 10-20 seconds depending on disk I/O and GPU bandwidth.

## TECHNICAL SPECIFICATIONS

- Model: F5-TTS (Diffusion Transformer)
- Interface: Gradio (Injected into WebView)
- Memory Footprint: ~6GB VRAM (Inference) + ~150MB System RAM (Tauri Wrapper)
- Security: Custom CSP defined in tauri.conf.json to allow localhost WebSocket/Blob traffic.

INTERNAL TECHNICAL DATA:

- GFX_VER: 11.0.0
- PORT: 7860
- HOST: 0.0.0.0
- POLL_INTERVAL: 1s
- TIMEOUT: 60s
