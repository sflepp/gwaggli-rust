# gwaggli-rust

gwaggli-rust is a playground project to discover the Rust programming language.

## Roadmap

- [x] Add basic WAVE / RIFF file reader for voice processing
- [x] Integrate OpenAI whisper for voice transcription
- [ ] Integrate microphone stream for voice transcription
- [ ] Add real-time voice transcription

## Dependencies

### Rust

gwaggli-rust is written in Rust. You can install Rust with [rustup](https://rustup.rs/).

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Nvidia graphics card with CUDA support

CUDA is enabled by default for maximum performance. Please install the latest [CUDA Toolkit](https://developer.nvidia.com/cuda-downloads) for your platform.

```
sudo apt-get install nvidia-cuda-toolkit
```

If you want to build gwaggli-rust without CUDA support, please disable the `cuda` feature in `Cargo.toml`.

### ALSA (Advanced Linux Sound Architecture)

ALSA is used for interaction with the audio hardware. Please install the ALSA development libraries for your platform.

```
sudo apt-get install libasound2-dev
```
