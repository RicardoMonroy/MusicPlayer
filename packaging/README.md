# Packaging and releases

## Cut a release

1. Set the same version in the workspace `Cargo.toml`, `apps/desktop/package.json`, and the desktop `tauri.conf.json`. Update `pkgver` in `packaging/arch/PKGBUILD` and regenerate `.SRCINFO` with `cd packaging/arch && makepkg --printsrcinfo > .SRCINFO`.
2. Commit the release changes, then tag and push from the repository root:
   ```sh
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. The Release workflow checks versions and uploads artifacts to a **draft** GitHub release. Inspect every job, install and test the artifacts on clean machines, and publish the draft manually. A failed job can leave a partial draft; rerun failed jobs before publishing.

The workflow follows CI's libmpv installation per OS. Rust, Bun, actions, Homebrew packages, and the latest upstream Windows mpv development archive are not pinned; rebuilding a tag may produce different binaries.

## Artifacts and runtime dependencies

- **Arch package (`musicplayer`)**: builds both interfaces from this repository's `v$pkgver` Git tag. Installs `/usr/bin/musicplayer`, `/usr/bin/musicplayer-tui`, desktop entry, hicolor icons, and GPL license. Requires `mpv`, `yt-dlp`, `webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `dbus`, and `openssl`. Build dependencies are Rust, Cargo, Bun, and Git. Build using `makepkg -si` in `packaging/arch`; fetching the tag and uncached Cargo/Bun dependencies requires network. Git sources use `SKIP` checksums; the tag is not signature-verified.
- **Linux `.deb` (x86_64)**: desktop app and Tauri resources. Declares `libmpv2` and `yt-dlp` alongside Tauri's generated system dependencies. Requires WebKitGTK 4.1, GTK3, D-Bus, and AppIndicator support. Built on Ubuntu 24.04; older distributions are not guaranteed compatible.
- **Linux `.AppImage` (x86_64)**: desktop app plus Tauri's AppImage runtime/resources. Requires host libmpv and its dependencies, working GTK/WebKit and desktop services, and FUSE support (or AppImage extraction). This workflow does not promise a fully portable AppImage.
- **Windows NSIS installer (x86_64)**: desktop app, resources, and `libmpv-2.dll` from the same mpv development archive used for linking. The resource mapping installs the DLL in the application directory beside the executable. WebView2 must be available; Tauri's default installer bootstrapper can install it. No separate mpv installation should be needed, but validate DLL loading on a clean Windows machine.
- **macOS `.dmg` (runner-native architecture)**: desktop `.app` and resources. **Known gap: not self-contained.** It links to Homebrew mpv and its transitive dylibs; install `brew install mpv` on a matching architecture at the same Homebrew prefix as the build runner. Copying only libmpv into `bundle.macOS.frameworks` would leave transitive Homebrew paths unresolved. No partial dylib copy or loader-path workaround is attempted. A future self-contained release needs recursive dependency bundling, install-name rewriting, and clean-machine validation before signing. The workflow does not configure Developer ID signing or notarization; Gatekeeper may block the app.
- **`musicplayer-tui-vVERSION-linux-x86_64.tar.gz`**: terminal binary, GPL license, and this README. Extract and place `musicplayer-tui` on PATH. Requires host libmpv, D-Bus, OpenSSL, their dependencies, and a compatible glibc (built on Ubuntu 24.04). No shared libraries are included.

**On every OS, install `yt-dlp` separately and ensure it is on the running application's PATH for YouTube playback.** Arch and Debian packages declare it as a dependency; the other artifacts do not include it. Desktop launches, especially on macOS, may inherit a different PATH from an interactive terminal.

## Validation and known gaps

Offline recipe checks:

```sh
cd packaging/arch
makepkg --printsrcinfo > .SRCINFO
bash -n PKGBUILD
namcap PKGBUILD  # if installed
```

Generating `.SRCINFO` does not fetch sources or compile the package. The release workflow, full Arch build, installer contents, clean-machine startup/playback, Windows DLL loading, and macOS dylib loading must be verified on their respective systems. The workflow does not configure Windows code signing. Bundled third-party libraries retain their own licenses; review their license notices and corresponding-source obligations before publishing the draft release.
