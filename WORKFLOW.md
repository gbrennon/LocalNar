# Replication Guide: Local Model + pi Agent

Step-by-step instructions to replicate the setup and use a local model as a pi coding agent. Tested on RTX 5070 12 GB with Qwen3-8B.

---

## 0. Prerequisites (one-time)

### 0.1 Clone and build llama.cpp

```sh
cd ~/repos/gbrennon
git clone --depth 1 https://github.com/ggerganov/llama.cpp.git llamacpp
cd llamacpp
cmake -B build -DGGML_VULKAN=ON -DCMAKE_BUILD_TYPE=Release
cmake --build build --target llama-server -j4
```

### 0.2 System dependencies

```sh
sudo dnf install -y vulkan-headers vulkan-loader-devel glslc spirv-headers-devel glslang-devel
```

### 0.3 Install pi extension

```sh
pi install git:github.com/huggingface/pi-llama
```

### 0.4 Adjust pi compaction

Edit `~/.pi/agent/settings.json`, set:

```json
"compaction": {
    "enabled": true,
    "keepRecentTokens": 12288,
    "reserveTokens": 8192
}
```

---

## 1. Download a model

```sh
# Recommended: Qwen3-8B (5 GB, fits 12 GB VRAM at 32K context)
mkdir -p ~/models/qwen3-8b
curl -L -o ~/models/qwen3-8b/Qwen3-8B-Q4_K_M.gguf \
  "https://huggingface.co/unsloth/Qwen3-8B-GGUF/resolve/main/Qwen3-8B-Q4_K_M.gguf"
```

Alternative if `hf` works:
```sh
hf download unsloth/Qwen3-8B-GGUF Qwen3-8B-Q4_K_M.gguf --local-dir ~/models/qwen3-8b/
```

---

## 2. Start the server

```sh
cd ~/repos/gbrennon/bare-ai-server

# For Qwen3 models (needed to prevent empty responses):
REASONING=off CTX_SIZE=32768 ./run-server.sh qwen3-8b

# For non-Qwen3 models:
CTX_SIZE=32768 ./run-server.sh <model-name>
```

Verify:
```sh
curl http://127.0.0.1:8080/health          # {"status":"ok"}
curl http://127.0.0.1:8080/v1/models       # shows model id
```

---

## 3. Run pi — one task at a time

### Option A: Single-task script (recommended)

```sh
cd /path/to/your/project

# Each invocation auto-kills old pi, ensures server is up, runs one task:
./pi-task.sh "Write Cargo.toml with actix-web and serde dependencies"
./pi-task.sh "Write src/main.rs with an actix-web server on port 3000"
./pi-task.sh "Write src/models.rs with a Task struct"
```

### Option B: Interactive (stay in pi)

```sh
cd /path/to/your/project
./fresh-context.sh
```

Then inside pi:
```
Write src/main.rs with an actix-web server
/fresh            ← saves progress to PROGRESS.md
/compact          ← frees context (summarizes old messages)
Write src/models.rs with a Task struct
```

---

## 4. The golden rules

### Rule 1: One module per prompt

**Never** ask for everything at once. Split into:

```
Task 1: build config (build.sbt / Cargo.toml)
Task 2: domain entities / models
Task 3: use cases / services
Task 4: adapters / repositories
Task 5: routes / controllers / entry point
```

### Rule 2: Always say "Use the write tool"

Without this, the model dumps code as chat prose instead of writing files.

### Rule 3: Add the concise prompt

Both `pi-task.sh` and `fresh-context.sh` inject this via `--append-system-prompt`:

```
Be concise. Do not explain your reasoning. Use tools immediately without narration.
```

Without it, the model generates 500-word reasoning essays before every action, eating context.

### Rule 4: Kill pi between large modules

For 4+ file modules, use `./pi-task.sh` (auto-kills) or manually:
```sh
pkill -9 -f 'node.*pi'
```

---

## 5. Verifying it works

### 5.1 Server-side tool calling

```sh
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model":"Qwen3-8B-Q4_K_M",
    "messages":[{"role":"user","content":"What is the weather in Paris?"}],
    "tools":[{"type":"function","function":{"name":"get_weather",
      "parameters":{"type":"object","properties":{"location":{"type":"string"}},"required":["location"]}}}],
    "tool_choice":"auto","max_tokens":64
  }'
```

Expected: `tool_calls` array with `name:"get_weather"` — NOT text in `content`.

### 5.2 pi end-to-end

```sh
cd /tmp && mkdir test-project && cd test-project
~/repos/gbrennon/bare-ai-server/pi-task.sh "Write hello.txt with the text: pi-task.sh verified"
cat hello.txt
# Expected: pi-task.sh verified
```

---

## 6. Troubleshooting

| Symptom | Fix |
|---|---|
| Empty chat responses | Start server with `REASONING=off` |
| Model writes 500-word essays before acting | Use `--append-system-prompt concise-prompt.txt` |
| "Context size exceeded" | Split into smaller tasks, one file per prompt |
| "peg-native format" error | Use Qwen3-8B or add `SKIP_CHAT_PARSING=1 CHAT_TEMPLATE=chatml` |
| pi hangs after "Done after 1 turn(s)" | Harmless auto-update crash; kill pi and retry |
| Server OOM | Reduce `CTX_SIZE` or use `KV_OFFLOAD=0` |
| pi launch times out | Use `(nohup pi ... </dev/null >/tmp/log 2>&1 &)` |
| `-p` prints text, no files | Add "Use the write tool" to prompt |

### Model overthinking example

**Without** concise prompt:
```
Okay, the user wants to initialize a simple Rust application. Let me think about
how to approach this. First, I need to create a new Cargo project. The standard
command is cargo new. They mentioned a simple app, so probably a binary project.
I'll use cargo new my_app which creates a new directory... [300 more words]
```

**With** concise prompt:
```
$ cargo new my_app && cd my_app && cargo add tokio
```

---

## 7. Files reference

| File | Purpose |
|---|---|
| `pi-task.sh` | One-shot task: kill pi + ensure server + run prompt + exit |
| `fresh-context.sh` | Kill pi, open fresh interactive pi session |
| `concise-prompt.txt` | Injected system prompt to suppress over-reasoning |
| `run-server.sh` | llama-server launcher (env vars: CTX_SIZE, REASONING, KV_OFFLOAD) |
| `register-with-pi.sh` | Register repo as pi project |
| `WORKFLOW.md` | This replication guide |
| `README.md` | Full setup reference with all flags |
| `PROGRESS.md` | Auto-generated by `/fresh` skill |
| `~/.pi/agent/skills/fresh/SKILL.md` | `/fresh` slash command definition |
