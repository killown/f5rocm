FROM rocm/pytorch:latest

USER root
ARG DEBIAN_FRONTEND=noninteractive

# 1. System Stack
RUN apt-get update && apt-get install -y \
    wget curl git ffmpeg sox libsox-fmt-all libsndfile1-dev \
    openssl libssl-dev build-essential aria2 tmux vim \
    openssh-server libsox-fmt-mp3 \
    librdmacm1 libibumad3 librdmacm-dev libibverbs1 libibverbs-dev ibverbs-utils ibverbs-providers \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 2. Pull your patched branch
RUN git clone -b infer_gradio_rocm https://github.com/killown/F5-TTS.git . \
    && git submodule update --init --recursive

# 3. Force Installation Logic
# We uninstall any existing f5-tts first to clear the path
RUN python3 -m pip uninstall -y f5-tts || true
RUN python3 -m pip install --no-cache-dir --break-system-packages \
    librosa pydub soundfile click gradio cached_path numpy tqdm transformers accelerate -e .

# 4. ROCm Compatibility Fixes
RUN python3 -m pip uninstall -y torchcodec bitsandbytes
ENV TORCHAUDIO_BACKEND=ffmpeg
ENV TORCH_ROCM_AOTRITON_ENABLE_EXPERIMENTAL=1
ENV SHELL=/bin/bash

# 5. CRITICAL: Force Python to look at /app first 
ARG PYTHONPATH
ENV PYTHONPATH="/app/src:${PYTHONPATH}"

EXPOSE 7860

CMD ["python3", "src/f5_tts/infer/infer_gradio.py", "--port", "7860", "--host", "0.0.0.0"]
