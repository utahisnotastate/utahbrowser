# Build troubleshooting (Windows)

If `install.ps1` fails while compiling Rust code, use this guide.

## Qdrant: Docker not required

Utah Browser installs the official [Qdrant](https://github.com/qdrant/qdrant) Windows binary automatically. On first run you need **internet once** for the download (~50MB). Data stays in `%LOCALAPPDATA%\UtahBrowser\qdrant`.

```powershell
.\scripts\Ensure-Qdrant.ps1
```

If native install fails, the installer tries Docker (optional). [Utahnetes](https://github.com/utahisnotastate/utahnetes) is a separate LAN swarm project and does **not** replace Qdrant for notebook search.

## Quick fix (try first)

Run from the **project root** (not only the `scripts` folder):

```powershell
cd C:\code\utahbrowser
.\scripts\Repair-BuildEnvironment.ps1
```

Then install again:

```powershell
.\scripts\install.ps1 -KnowledgePath "C:\path\to\notebooks"
```

Repair-only via installer:

```powershell
.\scripts\install.ps1 -RepairOnly
```

## Error: `can't find crate for zerovec_derive`

**Cause:** Corrupted or incomplete Cargo download cache.

**Fix:** The installer now runs `cargo clean` and refreshes affected crates automatically. If it persists:

```powershell
.\scripts\Repair-BuildEnvironment.ps1
```

## Error: `Application Control policy has blocked` (os error 4551)

**Cause:** Windows security (Defender, Smart App Control, or school/work policy) blocked Cargo’s `build-script-build.exe` files under `target\release\build\`.

**Fix:**

1. Open **PowerShell as Administrator** and run:

```powershell
cd C:\code\utahbrowser
.\scripts\Repair-BuildEnvironment.ps1 -AddExclusions
```

2. Or add exclusions manually in **Windows Security** → **Virus & threat protection** → **Exclusions** → **Add an exclusion** → **Folder**:

   - `C:\code\utahbrowser` (your clone path)
   - `C:\Users\<you>\.cargo`

3. On managed PCs, ask IT to allow build scripts under:

   - `<project>\target\release\build\`

4. Re-run:

```powershell
.\scripts\install.ps1
```

## Run install from the correct folder

Always use the repo root:

```powershell
cd C:\code\utahbrowser
.\scripts\install.ps1
```

Running only from `scripts\` works, but `cd` to the root avoids path confusion.

## Still stuck?

- Confirm Rust: `rustc --version` and `cargo --version`
- Full log: `target\install-build.log`
- Technical details: [Technical manual](../technical/MANUAL.md)
