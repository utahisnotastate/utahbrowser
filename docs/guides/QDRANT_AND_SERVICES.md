# Sovereign Intelligence Stack: Ollama & Qdrant

**The intelligence of the Utah Browser is entirely localized.** 

We utilize a combination of high-performance vector search and large language model inference to power the Truth Guard and Sovereign History Oracle. This guide explains how these services are orchestrated without cloud dependencies.

## 1. The Stack Overview

| Service | Protocol | Role | Source |
|---------|----------|------|--------|
| **Ollama** | REST | Embedding Generation & Synthesis | [ollama.com](https://ollama.com/) |
| **Qdrant** | REST/gRPC | Local Vector Search Engine | [qdrant.tech](https://qdrant.tech/) |

Both services must be active for the **Truth-Lens** to function. Utah Browser automatically manages these lifecycles.

---

## 2. Zero-Dependency Qdrant (Native)

Unlike standard implementations that require Docker, Utah Browser utilizes a **Native Windows Bootstrap**:

1. **Auto-Detection:** The browser pings `localhost:6333` on startup.
2. **Bootstrap:** If offline, the **Zero-Click Kernel** downloads the SOTA Qdrant binary directly from GitHub.
3. **Execution:** The service is launched as a hidden background process with its own PID management.
4. **Initialization:** The `utah_notebooks` collection is created automatically using optimized distance metrics (Cosine Similarity).

### Data Sovereignty Location:
All indexed knowledge is stored in:
`%LOCALAPPDATA%\UtahBrowser\qdrant\storage\`

---

## 3. Ollama Integration

Ollama provides the neural power for the Utah Browser. The following models are automatically pulled and managed by the installation kernel:

- **`nomic-embed-text`**: Used for high-dimensional vectorization of your browsing history and notebooks.
- **`llama3`**: Used for the Sovereign History Oracle chat and complex statement verification.

To verify status:
```powershell
ollama list
```

---

## 4. Lifecycle Orchestration

The browser manages services through three distinct layers:
1. **The Kernel:** `scripts/install.ps1` performs the initial setup and model pulling.
2. **The Launcher:** `Launch-UtahBrowser.ps1` runs a health check before opening the UI.
3. **The IPC Nexus:** The Rust backend continuously monitors service health, attempting auto-recovery if a service drops.

---

## 5. Manual Service Management

If you need to manage services manually for debugging or enterprise configuration:

```powershell
# Re-initialize only the Qdrant service
.\scripts\Ensure-Qdrant.ps1

# Check the health of the entire intelligence stack
.\scripts\Ensure-Services.ps1
```

---

## 6. Troubleshooting

| Symptom | Resolution |
|---------|------------|
| **"Oracle Offline"** | Open the Ollama application or run `ollama serve`. |
| **"History Index Error"** | Check `%LOCALAPPDATA%\UtahBrowser\qdrant\qdrant.err.log`. |
| **Connection Timeout** | Ensure port 6333 and 11434 are not blocked by system firewalls. |

---

**Mastery Log:** Intelligence stack documentation synchronized. Sovereignty is maintained.
