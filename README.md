# OVERVIEW
Tauri-based control plane designed to manage the lifecycle of an F5-TTS inference stack on ROCm (AMD) hardware. 
It automates the transition from system boot to a functional high-performance TTS environment by managing Docker containers and VRAM availability.

This project uses the following Docker image: 

### Docker Usage

```bash
docker pull killown/f5rocm:latest

docker run -it --rm \
    --device=/dev/kfd --device=/dev/dri \
    -p 7860:7860 \
    killown/f5rocm:latest
```

<img width="1911" height="1031" alt="image" src="https://github.com/user-attachments/assets/5d850ab6-4f8b-4631-9562-7969bcd92818" />
