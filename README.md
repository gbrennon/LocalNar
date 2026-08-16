# bare-ai-server

Local LLM serving with [llama.cpp](https://github.com/ggerganov/llama.cpp), built from source without containers. GPU offload via **Vulkan** on NVIDIA RTX 5070 (12 GB VRAM).

This guide covers everything from setup to using the local model as a coding agent with [pi](https://pi.dev).

---

## 1. Setup (one-time)

Clone with submodules (llama.cpp is pinned as a submodule):

```
git clone --recursive ssh://git@codeberg.org/gbrennon/bare-ai-server.git
```

If you already cloned without `--recursive`, or the submodule is out of date:

```sh
git submodule update --init --recursive
```

### 1.1 Build llama.cpp from the submodule

The submodule lives at `llama.cpp/`. Build it with Vulkan support:

```sh
cd llama.cpp
cmake -B build -DGGML_VULKAN=ON -DCMAKE_BUILD_TYPE=Release
cmake --build build --target llama-server -j4
```

The binary lands at `llama.cpp/build/bin/llama-server`. This repo's `run-server.sh` resolves it relative to itself (`../llama.cpp/build/bin/llama-server`).

### 1.2 System dependencies (already installed)

```sh
sudo dnf install -y vulkan-headers vulkan-loader-devel glslc spirv-headers-devel glslang-devel
```

### 1.3 Install pi-llama extension and register the project

```sh
pi install git:github.com/huggingface/pi-llama
./register-with-pi.sh
```

### 1.4 Adjust pi compaction settings

Edit `~/.pi/agent/settings.json`:

```json
"compaction": {
    "enabled": true,
    "keepRecentTokens": 12288,
    "reserveTokens": 8192
}
```

This triggers compaction earlier, leaving more headroom for model output in a 32K context window.

---

## 2. Models

### 2.1 Recommended model for 12 GB VRAM

| Model | Size (Q4_K_M) | Context | VRAM (32K ctx) | Tool calling |
|---|---|---|---|---|
| **Qwen3-8B** (unsloth) | 5.0 GB | 32K | ~10 GB | Best for 12 GB |
| Qwen2.5-7B | 4.4 GB | 32K | ~9 GB | OK |
| Qwen3-14B | 8.4 GB | 16K max | OOM at 32K | Best quality, tight |

Qwen3-8B is the sweet spot: newest generation, good tool calling, fits comfortably.

### 2.2 Download a model

```sh
# Primary method (if working):
hf download unsloth/Qwen3-8B-GGUF Qwen3-8B-Q4_K_M.gguf --local-dir ~/models/qwen3-8b/

# Fallback if hf is broken (Python 3.14 / typer incompatibility):
curl -L -o ~/models/qwen3-8b/Qwen3-8B-Q4_K_M.gguf \
  "https://huggingface.co/unsloth/Qwen3-8B-GGUF/resolve/main/Qwen3-8B-Q4_K_M.gguf"
```

---

## 3. Starting the server

```sh
cd ~/repos/gbrennon/bare-ai-server
CTX_SIZE=32768 ./run-server.sh qwen3-8b
```

Key env vars:

| Variable | Default | Use |
|---|---|---|
| `CTX_SIZE` | 8192 | **Always set to 32768 for pi** |
| `PORT` | 8080 | Override if port is busy |
| `HOST` | 0.0.0.0 | Bind address. Default exposes the server to the LAN so other machines can use it |
| `SKIP_CHAT_PARSING` | empty | Set to `1` only if you get PEG parser errors |
| `CHAT_TEMPLATE` | empty | Set to `chatml` for Mistral, leave empty for Qwen/Llama |
| `KV_OFFLOAD` | 1 | Set to `0` for `--no-kv-offload` (KV on CPU, frees VRAM for bigger models) |

The server binds to `0.0.0.0` by default and prints the LAN URL on startup:

```
Serving: /home/gbrennon/models/qwen3-8b/Qwen3-8B-Q4_K_M.gguf
  local:   http://127.0.0.1:8080
  network: http://192.168.0.14:8080  (reachable by other machines)
```

Point other machines on the same network at the `network` URL to consume the model. To restrict to this machine only, set `HOST=127.0.0.1`.

### 3.1 Verify the server

```sh
curl http://127.0.0.1:8080/health                           # {"status":"ok"}
curl http://127.0.0.1:8080/v1/models | jq '.data[].id'      # model alias
```

---

## 4. Verifying tool calling (curl test)

```sh
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model":"Qwen3-8B-Q4_K_M",
    "messages":[{"role":"user","content":"What is the weather in Paris?"}],
    "tools":[{
      "type":"function",
      "function":{
        "name":"get_weather",
        "parameters":{"type":"object","properties":{"location":{"type":"string"}},"required":["location"]}
      }
    }],
    "tool_choice":"auto",
    "max_tokens":64
  }'
```

Expected: `tool_calls` array with `name:"get_weather"` and `arguments:{"location":"Paris"}` — NOT text in `content`.

---

## 5. Using pi as a coding agent with the local model

### 5.1 The golden rule: fresh context every step

**Never ask the model to do everything in one prompt.** It will either overflow context (prose dump) or freeze.

Work **one module at a time**, killing pi between steps:

```sh
# Step 1: build.sbt
cd /path/to/project
pi --provider llama-cpp --model Qwen3-8B-Q4_K_M -a -p "Write build.sbt with multi-module sbt config..."

# Step 2: domain module (kill pi first)
pkill -9 -f 'node.*pi'
pi --provider llama-cpp --model Qwen3-8B-Q4_K_M -a -p "Write domain files: Entity.scala, Repository.scala..."

# Step 3: application module
pkill -9 -f 'node.*pi'
pi --provider llama-cpp --model Qwen3-8B-Q4_K_M -a -p "Write application use cases using domain layer..."

# Step 4: infrastructure module
pkill -9 -f 'node.*pi'
pi --provider llama-cpp --model Qwen3-8B-Q4_K_M -a -p "Write infrastructure adapter implementing domain port..."

# Step 5: presentation module
pkill -9 -f 'node.*pi'
pi --provider llama-cpp --model Qwen3-8B-Q4_K_M -a -p "Write presentation routes and Main.scala..."
```

### 5.2 Why this works

- Each step has a **clean 32K context window** — no history accumulation
- Small, focused prompts — the model doesn't get overwhelmed
- `-a` auto-approves tool calls (write, bash, edit)
- `-p` runs one pass and exits
- Killing pi between steps ensures truly fresh sessions

### 5.3 Prompt template for each step

A good step prompt contains 4 things:

1. **What already exists** (brief, so the model doesn't hallucinate)
2. **Exact file paths** to create
3. **What the file should do**
4. **"Use the write tool"** — this is critical, never omit it

Example:
```
Write one file: domain/src/main/scala/com/example/domain/Task.scala.
A case class with id: UUID, title: String, status: String. Use write tool.
```

### 5.4 When a step fails

If pi says "Context size has been exceeded" or "The model produced output that does not match":

1. The model tried to generate too much in one turn
2. **Split the step further** — one file per prompt instead of two
3. Kill pi and retry with a smaller ask

---

## 6. Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| "Context size exceeded" | Prompt too big, model output too long | Split into smaller steps (one file per prompt) |
| "peg-native format" error | Model outputs code as prose, not tool calls | Add `SKIP_CHAT_PARSING=1` to server, or use Qwen3-8B |
| Empty chat responses | Model stuck in thinking mode | Start server with `--reasoning off` flag |
| Server OOM | Model + KV cache too big | Reduce `CTX_SIZE` or use `KV_OFFLOAD=0` |
| pi says "Unknown provider llama-cpp" | Extension not installed | `pi install git:github.com/huggingface/pi-llama` |
| `-p` just prints text, no files | Model described code instead of writing | Add "Use write tool" to prompt, use smaller steps |
| pi hangs after "Done after 1 turn(s)" | Auto-update extension crash | Harmless — work completed; kill pi and move on |
| pi launch times out in shell | Shell kills the process group | Use `(nohup pi ... </dev/null >/tmp/out.log 2>&1 &)` |

### 6.1 The "peg-native" problem

Some models output code in prose instead of structured tool calls. When this happens:

1. Try **Qwen3-8B** — newer generation, much better at tool calls
2. If stuck with an older model, start the server with `SKIP_CHAT_PARSING=1` and `CHAT_TEMPLATE=chatml`
3. Split tasks into the smallest possible steps

### 6.2 If the CLI launch times out

The `setsid ... & disown` approach sometimes gets killed by shell timeouts. Use this pattern instead:

```sh
cd /path/to/project
(nohup pi --provider llama-cpp --model Qwen3-8B-Q4_K_M -a -p "prompt here" </dev/null >/tmp/pi-out.log 2>&1 &)
```

Then check `/tmp/pi-out.log` and the directory for files.

---

## 7. Quick reference

```sh
# Start server (keep this running in a terminal)
cd ~/repos/gbrennon/bare-ai-server && CTX_SIZE=32768 ./run-server.sh qwen3-8b

# Verify
curl http://127.0.0.1:8080/health

# Launch a coding session
cd /path/to/your/project
pi --provider llama-cpp --model Qwen3-8B-Q4_K_M -a -p "write file X at path Y using the write tool"

# Kill between steps for fresh context
pkill -9 -f 'node.*pi'

# Monitor resources
nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader
free -h | head -2
```

---

## Files in this repo

| File | Purpose |
|---|---|
| `llama.cpp/` | Pinned llama.cpp submodule (built with `GGML_VULKAN=ON`) |
| `run-server.sh` | Launcher (env vars: CTX_SIZE, PORT, HOST, SKIP_CHAT_PARSING, CHAT_TEMPLATE, KV_OFFLOAD) |
| `register-with-pi.sh` | Registers repo as a pi project |
| `README.md` | This guide |
