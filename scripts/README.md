# LocalNar Scripts

Run a local llama.cpp server and use it as a pi coding agent. No containers, no API keys. GPU + system RAM split support. Servers bind to `0.0.0.0` and print the LAN URL on startup, so other machines on the network can consume the model (e.g. `http://192.168.0.14:8080`).
## Quick start

```sh
# 1. Start the server (keep this terminal open)
REASONING=off CTX_SIZE=32768 ./run-server.sh qwen3-8b

# 2. In another terminal, code with pi
cd /path/to/your/project
./pi-task.sh "write Cargo.toml with axum and tokio deps"
./pi-task.sh "write src/main.rs with axum router and /health endpoint"
```

---

## Scripts

### `run-server.sh` — Fast mode (KV on GPU)

Model weights and KV cache both on GPU. Fastest inference (~80 tok/s).

```sh
# 8B at 32K context:
REASONING=off CTX_SIZE=32768 ./run-server.sh qwen3-8b

# List available models:
./run-server.sh list
```

### `run-server-both.sh` — Split mode (KV in RAM)

Model weights on GPU, KV cache in system RAM. Frees VRAM for bigger models or higher context.

```sh
# 8B at 65K context (massive window):
REASONING=off CTX_SIZE=65536 ./run-server-both.sh qwen3-8b

# 14B at 32K context (better quality):
REASONING=off CTX_SIZE=32768 ./run-server-both.sh qwen3
```

### `pi-task.sh` — Single task runner

Runs one prompt with fresh context. Auto-kills old pi sessions, starts the server if needed.

```sh
cd /path/to/your/project
./pi-task.sh "write models.py with Task dataclass"
./pi-task.sh "write main.py with CLI loop"
./pi-task.sh "write requirements.txt empty"
```

Each invocation is a clean 32K window. Run one task at a time.

### `fresh-context.sh` — Interactive session

Kills old pi, opens a fresh interactive session with concise prompt injected.

```sh
./fresh-context.sh
```

Inside pi, use `/fresh` to save progress and `/compact` to free context without exiting.

---

## Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `CTX_SIZE` | 32768 | Context window size |
| `PORT` | 8080 | Server port |
| `HOST` | 0.0.0.0 | Bind address. Default exposes the server to the LAN |
| `REASONING` | off | Set to `off` for Qwen3 models (prevents empty responses) |
| `PI_MODEL` | auto-detect | Override model id for pi-task.sh |
| `SKIP_CHAT_PARSING` | empty | Set to `1` for Mistral models (PEG parser workaround) |
| `CHAT_TEMPLATE` | empty | Set to `chatml` for Mistral models |

---

## Model download

```sh
mkdir -p ~/models/qwen3-8b
curl -L -o ~/models/qwen3-8b/Qwen3-8B-Q4_K_M.gguf \
  "https://huggingface.co/unsloth/Qwen3-8B-GGUF/resolve/main/Qwen3-8B-Q4_K_M.gguf"
```

Models are stored in `~/models/`. The server auto-discovers them by scanning for `*.gguf` files.

---

## Verification

```sh
# Server health:
curl http://127.0.0.1:8080/health          # {"status":"ok"}

# Tool calling (OpenAI-compatible):
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"Qwen3-8B-Q4_K_M","messages":[{"role":"user","content":"Hi"}],"max_tokens":10}'

# pi end-to-end:
cd /tmp && rm -rf test && mkdir test && cd test
/path/to/scripts/pi-task.sh "write hello.txt with text: works"
cat hello.txt  # should print: works
```

---

## Troubleshooting

| Problem | Fix |
|---|---|
| Empty chat responses | `REASONING=off` in server command |
| Model writes essays before acting | Use `./pi-task.sh` (injects concise prompt) |
| Context size exceeded | One file per prompt, use `./pi-task.sh` per task |
| Server OOM | Use `./run-server-both.sh` (KV in RAM) |
| PEG parser errors | `SKIP_CHAT_PARSING=1 CHAT_TEMPLATE=chatml` |
| Wrong model | Set `PI_MODEL=Qwen3-14B-Q4_K_M` |
| pi says "Unknown provider llama-cpp" | `pi install git:github.com/huggingface/pi-llama` |
| pi hangs after "Done after 1 turn(s)" | Auto-update extension crash; work completed, kill pi and move on |
| pi launch times out in shell | Shell kills the process group: `(nohup pi ... </dev/null >/tmp/pi-out.log 2>&1 &)` |

---

## Choosing a model for your VRAM

Sizes are Q4_K_M; VRAM figures assume a 32K context.

| Model | Size | Context | VRAM (32K ctx) | Notes |
|---|---|---|---|---|
| **Qwen3-8B** (unsloth) | 5.0 GB | 32K | ~10 GB | Sweet spot for 12 GB |
| Qwen2.5-7B | 4.4 GB | 32K | ~9 GB | Older generation |
| Qwen3-14B | 8.4 GB | 16K max | OOM at 32K | Best quality, tight |

Qwen3-8B is the default recommendation: newest generation, good tool calling,
fits comfortably in 12 GB.

---

## Using pi effectively

### Fresh context every step

Never ask the model to do everything in one prompt. It will either overflow the
context and dump prose, or freeze. Drive one file per prompt, killing pi between
steps so each starts from a clean window - which is what `pi-task.sh` does for
you.

### What a good step prompt contains

1. What already exists, briefly, so the model does not invent it.
2. The exact file paths to create.
3. What the file should do.
4. "Use the write tool" - never omit this.

```
Write one file: domain/src/main/scala/com/example/domain/Task.scala.
A case class with id: UUID, title: String, status: String. Use write tool.
```

### When a step fails

"Context size has been exceeded" or "The model produced output that does not
match" both mean the model tried to generate too much in one turn. Split the step
further - one file per prompt - and retry with a smaller ask.

### pi compaction settings

In `~/.pi/agent/settings.json`, compacting earlier leaves more headroom for
output in a 32K window:

```json
"compaction": {
    "keepRecentTokens": 12288,
    "reserveTokens": 8192
}
```
