# Model validation guide

Instructions for validating and running models downloaded by LocalNar using
`llama-cli`, `llama-server`, or `llama-bench` across any infrastructure.

---

## 1. Locating installed models

LocalNar stores models in a structured on-disk hierarchy rooted at
`$LOCALNAR_MODELS_DIR` (defaults to `~/.cache/localnar/models`):

```text
~/.cache/localnar/models/
└── <owner>/
    └── <repository>/
        └── <revision>/
            ├── <model-file>.gguf
            └── <model-file>.gguf.sha256
```

### Finding the exact path

- **Inside LocalNar TUI**:
  1. Press `Tab` or `l` to switch to **Library** mode.
  2. Select the model using `↑` / `↓`.
  3. Press `i` or `Enter` to inspect the model. The exact absolute path to the
     `.gguf` file is shown on screen.
- **Via terminal**:
  ```sh
  find "${LOCALNAR_MODELS_DIR:-$HOME/.cache/localnar/models}" -name "*.gguf"
  ```

---

## 2. Integrity check before execution

Before loading a model into an inference engine, verify that the download is
complete and uncorrupted:

- **Inside LocalNar**: In Library mode, select the model and press `v`. The
  state badge will confirm **verified** against the upstream SHA-256 digest.
- **Via command line**:
  ```sh
  MODEL_PATH="/path/to/model.gguf"
  echo "$(cat "${MODEL_PATH}.sha256")  ${MODEL_PATH}" | sha256sum -c
  ```

If the verification passes (`OK`), the model weights are intact.

---

## 3. Hardware sizing and parameter calculation

Regardless of your system specs (NVIDIA, AMD, Apple Silicon, or pure CPU),
calculate your offload parameters before launching:

### A. GPU layers (`-ngl`)

| Scenario | Rule | Setting |
|---|---|---|
| **Model fits fully in VRAM** (e.g. 8B on 12 GB VRAM) | Offload all layers to GPU | `-ngl 99` |
| **Model larger than VRAM** (e.g. 27B on 12 GB VRAM) | Offload as many layers as fit safely in free VRAM | `-ngl <calculated>` |
| **No GPU / Pure CPU** | Keep all layers in system RAM | `-ngl 0` |

**Calculating hybrid layers for large models:**
$$\text{Layers to offload} \approx \frac{\text{Free VRAM (in MB)} - \text{Context buffer margin (~1500 MB)}}{\text{Model file size (in MB)} / \text{Total model layers}}$$

*Example*: A 27B model (15,000 MB, 65 layers) on a 12 GB GPU with ~10,800 MB free:
$$\frac{10800 - 1500}{15000 / 65} \approx \frac{9300}{230} \approx 40 \text{ to } 42 \text{ layers}$$

### B. CPU threads (`-t`)

Always set `-t` equal to your machine's **physical CPU cores** (not virtual/hyperthreaded cores). This prevents cache contention and SMT slowdown when layers run on CPU:

```sh
# On Linux, inspect physical core count:
lscpu -p | grep -v '^#' | sort -u -t, -k 2,4 | wc -l
```

### C. Context length and KV cache quantization

- `-c <tokens>`: Sets context window size (e.g. `4096`, `8192`, `16384`).
- `-fa on`: Enables Flash Attention (significantly reduces VRAM usage for context).
- `-ctk q8_0 -ctv q8_0`: Quantizes key/value cache to 8-bit, cutting KV memory consumption in half with negligible accuracy loss.

---

## 4. Validating with `llama-cli` (interactive prompt)

Run `llama-cli` for a direct terminal conversation to test generation speed and output correctness:

### Fully GPU-offloaded (Model fits in VRAM)

```sh
llama-cli \
  -m "/path/to/model.gguf" \
  -ngl 99 \
  -fa on \
  -c 8192 \
  -p "Explain the concept of entropy in simple terms:"
```

### Hybrid GPU + CPU offload (Model exceeds VRAM)

```sh
llama-cli \
  -m "/path/to/model.gguf" \
  -ngl 40 \
  -t 8 \
  -fa on \
  -c 8192 \
  -ctk q8_0 \
  -ctv q8_0 \
  -b 2048 \
  -ub 512
```

### Pure CPU execution

```sh
llama-cli \
  -m "/path/to/model.gguf" \
  -ngl 0 \
  -t 8 \
  -c 4096
```

---

## 5. Validating with `llama-server` (HTTP API)

Start a local OpenAI-compatible API server:

```sh
llama-server \
  -m "/path/to/model.gguf" \
  -ngl 40 \
  -t 8 \
  -fa on \
  -c 16384 \
  -ctk q8_0 \
  -ctv q8_0 \
  --port 8080 \
  --parallel 1
```

Once loaded (`model loaded`, `listening on http://127.0.0.1:8080`), validate via `curl`:

```sh
curl http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "user", "content": "Respond with one word: ready"}],
    "max_tokens": 10
  }'
```

---

## 6. Performance benchmarking with `llama-bench`

To benchmark throughput and distinguish compute throughput from memory bandwidth:

```sh
llama-bench \
  -m "/path/to/model.gguf" \
  -ngl 40 \
  -fa 1 \
  -p 512 \
  -n 128
```

- `-p 512 -n 0`: Benchmarks **prompt ingestion (prefill)** speed (tokens/sec). Ingesting prompt batches exercises parallel matrix cores (GPU compute utilization will approach 100%).
- `-p 0 -n 128`: Benchmarks **autoregressive token generation** speed (tokens/sec).

---

## 7. Troubleshooting common issues

### CUDA out of memory (`cudaMalloc failed`)

- **Cause**: `-ngl` was set higher than available VRAM, or context size (`-c`) allocated too large a KV buffer.
- **Fix**:
  1. Reduce `-ngl` by 2–4 layers.
  2. Lower `-c` (e.g. from `16384` to `8192`).
  3. Add `-ctk q8_0 -ctv q8_0` to compress the KV cache.
  4. Ensure Flash Attention is enabled (`-fa on`).

### GPU compute utilization reads 25%–35% in `nvidia-smi`

- **Cause**: This is normal behavior during **hybrid CPU/GPU execution**.
- **Explanation**: In an autoregressive model, layers execute sequentially for each token. When 40 layers run on GPU and 25 on CPU, the GPU finishes its 40 layers in milliseconds and then pauses idle while the CPU processes the remaining 25 layers in system RAM. The GPU active duty cycle is only ~25%–35% of total elapsed time.
- **Fix**: To achieve 100% GPU utilization during generation, the model must fit entirely in VRAM (`-ngl 99`).
