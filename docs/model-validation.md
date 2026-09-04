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
  1. Press `Alt+2` (or cycle with `Tab`) to switch to the **Library** screen.
     *(Screens use `Alt+1` for Search, `Alt+2` for Library, and `Alt+3` for Help).*
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

- **Inside LocalNar**: On the Library screen (`Alt+2` or `Tab`), select the model
  and press `v`. The state badge will confirm **verified** against the upstream
  SHA-256 digest.
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

> **Tip**: For scripted non-interactive execution, pass `-st` (or `--single-turn`)
> so the program outputs the generation and immediately exits instead of waiting
> for interactive user input in a conversation loop.

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

`llama-bench` strips away tokenizer, chat templating, and disk overhead to measure
pure hardware execution speed.

### Basic run

```sh
llama-bench \
  -m "/path/to/model.gguf" \
  -ngl 99 \
  -fa 1 \
  -p 512 \
  -n 128
```

Sample output:
```text
| model          | size     | params  | backend | ngl | fa | test  | t/s             |
| -------------- | -------- | ------- | ------- | --- | -- | ----- | --------------- |
| qwen2 14B Q4_0 | 7.93 GiB | 14.77 B | CUDA    | 99  | 1  | pp512 | 2906.89 ± 72.39 |
| qwen2 14B Q4_0 | 7.93 GiB | 14.77 B | CUDA    | 99  | 1  | tg128 |   60.47 ± 0.12  |
```

- **`pp512` (Prompt Processing / Prefill)**: Tokens/sec while ingesting a 512-token prompt.
  Runs dense matrix multiplications (GEMM), pushing GPU compute cores to **90%–100%**.
- **`tg128` (Text Generation / Decode)**: Tokens/sec while generating output tokens.
  Memory-bandwidth bound.

### Matrix benchmarking across context windows

Pass comma-separated values to test multiple context lengths in a single run, and
use `-o md` to output directly in Markdown format:

```sh
llama-bench \
  -m "/path/to/model.gguf" \
  -ngl 99 \
  -fa 1 \
  -p 512,1024,2048 \
  -n 0,128 \
  -o md
```

### Evaluating layer scaling (GPU vs. CPU split)

To measure the exact throughput degradation caused by leaving layers on the CPU:

```sh
llama-bench \
  -m "/path/to/model.gguf" \
  -ngl 0,20,40,99 \
  -fa 1 \
  -p 512 \
  -n 64
```

---

## 7. Quality evaluation with `llama-perplexity`

Perplexity (PPL) measures how accurately a model predicts text tokens. A **lower**
score indicates higher quality and less degradation from quantization.

### Testing with any local file

The `-f` flag requires a real text file on disk. You can evaluate perplexity immediately
using any local file (e.g. `README.md`):

```sh
llama-perplexity \
  -m "/path/to/model.gguf" \
  -f README.md \
  -ngl 99 \
  -c 512
```

Sample output:
```text
perplexity: calculating perplexity over 2 chunks, n_ctx=512, batch_size=2048
[1]11.3007,[2]10.5921,
Final estimate: PPL = 10.5921 +/- 1.45250
```

### Standard WikiText-2 benchmark

For standardized scores comparable against published model cards:

```sh
# 1. Download and extract the standard dataset
wget https://huggingface.co/datasets/ggml-org/ci/resolve/main/wikitext-2-raw-v1.zip
unzip wikitext-2-raw-v1.zip

# 2. Calculate standard perplexity
llama-perplexity \
  -m "/path/to/model.gguf" \
  -f wikitext-2-raw/wiki.test.raw \
  -ngl 99 \
  -c 2048
```

---

## 8. Speculative decoding and companion tools

### Accelerating generation with speculative decoding (`-md`)

When running a 14B or 27B model, you can pair it with a tiny draft model (e.g. 0.5B
or 1.5B) to boost generation speed from ~60 t/s up to **90–110+ t/s**:

```sh
llama-cli \
  -m "/path/to/base-model-14B.gguf" \
  -md "/path/to/draft-model-0.5B.gguf" \
  -ngl 99 \
  -ngld 99 \
  -fa on
```

### Other utilities in the suite

| Tool | Purpose | Typical Command |
|---|---|---|
| **`llama-tokenize`** | Inspect token decomposition and count prompt tokens | `llama-tokenize -m <model.gguf> -p "Sample text"` |
| **`llama-embedding`** | Generate vector embeddings for RAG or search | `llama-embedding -m <model.gguf> -p "Search query"` |
| **`llama-simple-chat`** | Minimal chat interface with zero TUI dependencies | `llama-simple-chat -m <model.gguf> -ngl 99` |
| **`llama-quantize`** | Convert unquantized (`F16`/`BF16`) weights to `Q4_K_M` or `IQ4_XS` | `llama-quantize <input.gguf> <output.gguf> Q4_K_M` |

---

## 9. Troubleshooting common issues

### CUDA out of memory (`cudaMalloc failed`)

- **Cause**: `-ngl` was set higher than available VRAM, or context size (`-c`) allocated too large a KV buffer.
- **Fix**:
  1. Reduce `-ngl` by 2–4 layers.
  2. Lower `-c` (e.g. from `16384` to `8192`).
  3. Add `-ctk q8_0 -ctv q8_0` to compress the KV cache.
  4. Ensure Flash Attention is enabled (`-fa on`).

### GPU reads ~25%–35% and CPU reads < 10%

A common point of confusion during hybrid execution is why **both** GPU and CPU report low utilization percentages in `nvidia-smi` and `top`/`htop`:

1. **The Sequential Ping-Pong (Duty Cycle)**:
   - Generating each token is strictly sequential.
   - The GPU computes layers 0–41 in ~40 ms. During this time, the CPU is completely idle (0% CPU).
   - The CPU computes layers 42–64 in ~120 ms. During this time, the GPU is completely idle (0% GPU).
   - Because both devices take turns waiting on each other, neither device can ever register a continuous 100% active duty cycle during generation.

2. **The CPU Memory-Bandwidth Wall**:
   - Why doesn't the CPU spike to 100% during its turn?
   - Single-token generation requires streaming ~5.5 GB of model weights from system DDR5 RAM to CPU cache once per token.
   - Dual-channel DDR5 bandwidth (~50–60 GB/s) saturates almost instantaneously.
   - Once the memory bus is saturated, the CPU execution units (ALUs/AVX cores) spend most clock cycles stalled waiting on memory cache lines.
   - Operating system metrics (`top`, `htop`) measure active instruction retirement across all logical threads (e.g. 16 threads on an 8-core CPU), so memory-stalled threads register as **< 10%–15% CPU utilization**.

3. **How to observe 90%+ GPU compute utilization**:
   - **Prompt Ingestion (Prefill)**: When feeding a large prompt (e.g. 1,000–2,000 tokens), the entire prompt is evaluated simultaneously via batch matrix-matrix multiplication (GEMM). During prefill, GPU tensor cores run at full parallelism, and GPU utilization will spike to **90%–100%**.
     ```sh
     # Test prompt prefill saturation with a large input:
     llama-cli -m "/path/to/model.gguf" -ngl 40 -fa on -c 8192 -p "$(seq -s ' ' 1000)"
     ```
   - **100% GPU Offload**: Use a model whose entire weight footprint fits inside VRAM (`-ngl 99`). This eliminates the CPU wait entirely, producing continuous 80%–100% GPU utilization during generation.

