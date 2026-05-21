# Build & install troubleshooting (Windows)

**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

---

## Run install from the repo root

```powershell
cd C:\code\utahbrowser
.\scripts\install.ps1 -KnowledgePath "C:\path\to\notebooks"
```

Running from `scripts\` works; `cd` to the root avoids confusion.

---

## Qdrant: native install (no Docker)

Utah Browser downloads [Qdrant](https://github.com/qdrant/qdrant) for Windows automatically.

```powershell
.\scripts\Ensure-Qdrant.ps1
```

| Path | Purpose |
|------|---------|
| `%LOCALAPPDATA%\UtahBrowser\qdrant\bin\qdrant.exe` | Server binary |
| `qdrant.out.log` / `qdrant.err.log` | Logs |

Full guide: [QDRANT_AND_SERVICES.md](QDRANT_AND_SERVICES.md)

### Error: RedirectStandardOutput and RedirectStandardError are the same

**Cause:** Older installer tried to log stdout and stderr to one file.

**Fix:** `git pull` to get the fix (separate `.out` and `.err` logs), then:

```powershell
.\scripts\Ensure-Qdrant.ps1
.\scripts\install.ps1 -KnowledgePath "C:\knowledgebase"
```

### Qdrant download or API fails

- Confirm internet access to `github.com`
- Check `qdrant.err.log` for port conflicts (6333 in use)
- Optional Docker fallback: [QDRANT_AND_SERVICES.md](QDRANT_AND_SERVICES.md)

---

## Rust build errors

### Quick fix

```powershell
cd C:\code\utahbrowser
.\scripts\Repair-BuildEnvironment.ps1
.\scripts\install.ps1 -KnowledgePath "C:\path\to\notebooks"
```

Repair-only:

```powershell
.\scripts\install.ps1 -RepairOnly
```

### `can't find crate for zerovec_derive`

Corrupted Cargo cache. The installer auto-runs `cargo clean` and retries. If needed:

```powershell
.\scripts\Repair-BuildEnvironment.ps1
```

### Application Control policy blocked (os error 4551)

Windows blocked `build-script-build.exe` under `target\release\build\`.

1. Administrator PowerShell:

```powershell
.\scripts\Repair-BuildEnvironment.ps1 -AddExclusions
```

2. Or add Defender exclusions for your repo folder and `%USERPROFILE%\.cargo`

3. Re-run `.\scripts\install.ps1`

### PowerShell stops on "Downloading crates ..."

Cargo writes progress to stderr; fixed in current `scripts/kernel/Cargo.ps1`. `git pull` and retry.

---

## Cargo progress treated as error

Symptom: install exits right after `Downloading crates ...` with `NativeCommandError`.

**Fix:** Update to latest `main` (uses `Invoke-Cargo` wrapper).

---

## Still stuck?

- `rustc --version` and `cargo --version`
- Build log: `target\install-build.log`
- [Technical manual](../technical/MANUAL.md)
- [Installation](../INSTALLATION.md)
