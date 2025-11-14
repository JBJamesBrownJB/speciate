# Tauri GUI in Devcontainer - RESOLVED ✅

**Status:** RESOLVED (Nov 12, 2025)
**Solution:** X11 forwarding with GPU passthrough enabled

## Original Problem

Tauri desktop apps require a display server (X11/Wayland) to render the GUI window. Devcontainers typically run headless without a display, causing this error:

```
(speciate-app:11980): dbind-WARNING **: Couldn't connect to accessibility bus: Failed to connect to socket /run/user/1000/at-spi/bus_1: No such file or directory
Gtk-Message: Failed to load module "canberra-gtk-module"
```

**The app compiles and runs, but cannot open a window.**

## Solutions

### Option 1: Run Tauri on Local Machine (Recommended)

Develop the Rust simulation in the devcontainer, but run the Tauri GUI on your local machine:

```bash
# On your LOCAL machine (not in devcontainer)
cd apps/portal
npm install
npm run tauri:dev
```

**Pros:**
- Native GUI performance
- No display forwarding complexity
- Works on all platforms (Mac/Windows/Linux)

**Cons:**
- Need Rust toolchain installed locally
- May have dependency version mismatches

### Option 2: X11 Forwarding from Devcontainer

Forward the display from devcontainer to your host machine's X server.

**Requirements:**
- Mac: Install XQuartz
- Windows: Install VcXsrv or Xming
- Linux: X11 already available

**Setup:**
1. Start X server on host
2. Set `DISPLAY` environment variable in devcontainer
3. Add X11 socket volume mount to `.devcontainer/devcontainer.json`

**Pros:**
- Everything runs in devcontainer
- Consistent environment

**Cons:**
- Complex setup
- Performance overhead (network rendering)
- Platform-specific X server requirements

### Option 3: Headless Testing (Development Only)

Test the Rust simulation and IPC layer without the GUI:

```bash
# Test simulation logic
cd apps/simulation
cargo test

# Test Tauri commands (unit tests)
cd apps/portal/src-tauri
cargo test
```

**Pros:**
- Works in any environment
- Fast iteration for backend logic

**Cons:**
- Cannot test actual GUI/rendering
- Frontend integration untested

## Recommended Workflow for Sprint 7

1. **Backend development (Rust simulation):** Use devcontainer
   - ECS systems, A-Life logic, snapshot queue
   - Run `cargo test` for validation

2. **Frontend development (TypeScript + PixiJS):** Use devcontainer
   - Vite dev server works fine: `npm run dev`
   - Test rendering in browser at `http://localhost:5173`

3. **Tauri integration testing:** Use local machine
   - IPC bridge, desktop window, final integration
   - Run `npm run tauri:dev` locally

4. **Production builds:** Use local machine or CI/CD
   - `cargo tauri build` requires GUI libraries

## Implemented Solution

**Choice:** Option 2 (X11 Forwarding) with GPU passthrough

### Why X11 Forwarding?

After research and team discussion, we chose X11 forwarding because:
- ✅ True environment isolation (everything runs in container)
- ✅ No host dependencies except Docker + VSCode
- ✅ Simpler than expected on Linux (team's primary platform)
- ✅ GPU acceleration possible via `/dev/dri` passthrough
- ✅ Works on Windows 11 via WSLg (built-in)
- ✅ Enables 60-90 FPS rendering (required for PixiJS)

### Configuration Changes

**Files Modified:**
- `.devcontainer/docker-compose.yml` - Added X11 socket mount, GPU device, display environment variables
- `.devcontainer/Dockerfile` - Added X11 utilities (`x11-apps`, `mesa-utils`) and GPU drivers
- `.devcontainer/devcontainer.json` - Added Tauri VSCode extension, updated Rust analyzer config
- `.vscode/launch.json` - Added Tauri debugging configurations
- `.vscode/tasks.json` - Added `tauri:dev`, `ui:dev`, `ui:build`, `test:all` tasks

**Key Docker Configuration:**
```yaml
volumes:
  - /tmp/.X11-unix:/tmp/.X11-unix  # X11 socket
  - /dev/shm:/dev/shm  # Shared memory for performance
environment:
  - DISPLAY=${DISPLAY:-:0}
  - XDG_RUNTIME_DIR=/tmp
devices:
  - /dev/dri:/dev/dri  # GPU passthrough
```

### Host Setup Required

**Linux (Ubuntu):**
```bash
xhost +local:docker  # One-time per boot
```

**Windows 11 (WSL2 + WSLg):**
```bash
# No setup needed - WSLg provides X11/Wayland automatically
```

**macOS (XQuartz):**
```bash
brew install --cask xquartz
# Configure XQuartz to allow network connections
```

See [docs/development/HOST_SETUP.md](/workspace/docs/development/HOST_SETUP.md) for detailed platform-specific instructions.

## Verification Steps

Inside devcontainer terminal:

```bash
# Test X11 connection
xclock &  # Should show clock window on host desktop

# Test GPU acceleration
glxinfo | grep "direct rendering"  # Should show "Yes"

# Run Tauri
cd apps/portal
npm run tauri dev  # Should open Tauri window with 60+ FPS
```

## Current Status ✅

- ✅ X11 forwarding configured in devcontainer
- ✅ MIT-SHM workaround implemented (fixes BadAccess X11 error)
- ✅ GPU passthrough enabled for hardware acceleration (Intel/AMD)
- ✅ **NVIDIA GPUs fully configured and working** - Setup documented in [NVIDIA_GPU_SETUP.md](/workspace/docs/development/NVIDIA_GPU_SETUP.md)
- ✅ VSCode launch configurations for debugging
- ✅ Documentation complete (HOST_SETUP.md, DEVELOPMENT_WORKFLOW.md, NVIDIA_GPU_SETUP.md)
- ✅ README.md updated with devcontainer workflow
- ✅ **End-to-end tested: Tauri GUI opens successfully in devcontainer with GPU acceleration (Nov 12, 2025)**

### Known Issue: MIT-SHM X11 Error (FIXED ✅)

**Symptoms:**
```
(speciate-app:15214): Gdk-WARNING **: The program 'speciate-app' received an X Window System error.
The error was 'BadAccess (attempt to access private resource denied)'.
(Details: serial 355 error_code 10 request_code 130 (MIT-SHM) minor_code 1)
```

**Cause:** X11 server denies shared memory access from Docker container.

**Solution:** Disable MIT-SHM extension by setting `MIT_SHM_DISABLE=1` environment variable.

**Status:** ✅ Fixed in `tauri-dev.sh` script (automatically applied)

### Known Issue: NVIDIA GPU Driver Mismatch (FIXED ✅)

**Status:** ✅ Resolved - NVIDIA GPU acceleration is now fully operational.

**Previous Symptoms (now fixed):**
```
libGL error: failed to load driver: nouveau
libEGL warning: DRI2: failed to authenticate
OpenGL renderer string: llvmpipe (software rendering)
```

**Root Cause:** Host uses NVIDIA proprietary drivers, but container didn't have matching drivers.

**Solution Applied:** Followed [NVIDIA_GPU_SETUP.md](/workspace/docs/development/NVIDIA_GPU_SETUP.md) to install:
- NVIDIA Container Toolkit (host)
- Matching NVIDIA drivers (container)
- GPU passthrough via `--gpus=all` docker flag

**Current State:** Hardware GPU acceleration is working. If you experience low FPS, it's likely due to other bottlenecks (not GPU).

### Software Rendering Mode (Workaround)

If setting up NVIDIA drivers is too complex, use software rendering:

```bash
# Run Tauri with software rendering
export LIBGL_ALWAYS_SOFTWARE=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export GDK_GL=disable
cd apps/portal
npm run tauri dev
```

**Trade-offs:**
- ✅ Works immediately, no host setup
- ✅ Good enough for IPC/backend development
- ❌ Lower FPS (~15-20 instead of 60+)
- ❌ Can't test rendering performance realistically

## CI/CD Implementation

For GitHub Actions builds, use `xvfb` (virtual framebuffer):

```yaml
- name: Build Tauri
  run: |
    sudo apt-get install -y xvfb
    xvfb-run --auto-servernum cargo tauri build
```

**See:** `.github/workflows/ci.yml` (to be created in future sprint)

## References

- **Reference Implementation:** Community Tauri devcontainer config (confirmed X11 forwarding is standard approach)
- **Research:** frontend-fanny + architect-andy research (Nov 12, 2025)
- **Documentation:** [HOST_SETUP.md](/workspace/docs/development/HOST_SETUP.md), [DEVELOPMENT_WORKFLOW.md](/workspace/docs/development/DEVELOPMENT_WORKFLOW.md)
