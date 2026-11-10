# MusicPlayer

Reproductor de música para Windows, Linux y macOS, con una app de escritorio y una app de terminal (TUI) para Linux. Reproduce tu música local (MP3, FLAC y formatos comunes) y YouTube Music.

- **Escritorio**: vista completa, mini reproductor con skins, ecualizador, letras con traducción al español, cola de reproducción y navegación por artistas y álbumes como en YouTube Music.
- **Terminal (TUI)**: las mismas fuentes, cola, ecualizador, letras y portadas, desde la terminal.

## Descargas (v0.1.0)

| Sistema | Descarga | Notas |
|---|---|---|
| Windows 10/11 (x64) | [MusicPlayer_0.1.0_x64-setup.exe](https://github.com/RicardoMonroy/MusicPlayer/releases/download/v0.1.0/MusicPlayer_0.1.0_x64-setup.exe) | Instalador. Incluye mpv. |
| macOS (Apple Silicon) | [MusicPlayer_0.1.0_aarch64.dmg](https://github.com/RicardoMonroy/MusicPlayer/releases/download/v0.1.0/MusicPlayer_0.1.0_aarch64.dmg) | Requiere `mpv` de Homebrew. |
| Debian / Ubuntu (x64) | [MusicPlayer_0.1.0_amd64.deb](https://github.com/RicardoMonroy/MusicPlayer/releases/download/v0.1.0/MusicPlayer_0.1.0_amd64.deb) | Ubuntu 24.04 o más reciente. |
| Linux, cualquier distro (x64) | [MusicPlayer_0.1.0_amd64.AppImage](https://github.com/RicardoMonroy/MusicPlayer/releases/download/v0.1.0/MusicPlayer_0.1.0_amd64.AppImage) | Requiere `libmpv` instalado. |
| Arch Linux / Omarchy | [PKGBUILD](packaging/arch/PKGBUILD) | Compila e instala escritorio + TUI. |
| TUI Linux (x64) | [musicplayer-tui-v0.1.0-linux-x86_64.tar.gz](https://github.com/RicardoMonroy/MusicPlayer/releases/download/v0.1.0/musicplayer-tui-v0.1.0-linux-x86_64.tar.gz) | Solo la app de terminal. |

Todas las versiones: [página de releases](https://github.com/RicardoMonroy/MusicPlayer/releases).

**En todos los sistemas, YouTube Music necesita [`yt-dlp`](https://github.com/yt-dlp/yt-dlp)** instalado y en el `PATH`. La música local funciona sin él.

## Instalación

### Windows

1. Instala yt-dlp: `winget install yt-dlp.yt-dlp`
2. Descarga y abre `MusicPlayer_0.1.0_x64-setup.exe`.
3. El instalador no está firmado: si Windows SmartScreen lo bloquea, pulsa **Más información → Ejecutar de todas formas**.

Si falta WebView2, el instalador lo descarga.

### macOS (Apple Silicon)

1. Instala las dependencias con [Homebrew](https://brew.sh):
   ```sh
   brew install mpv yt-dlp
   ```
2. Descarga el `.dmg`, ábrelo y arrastra **MusicPlayer** a **Aplicaciones**.
3. La app no está firmada. Si macOS dice que no se puede abrir, ejecuta una vez:
   ```sh
   xattr -dr com.apple.quarantine /Applications/MusicPlayer.app
   ```

No hay versión para Mac con procesador Intel.

### Debian / Ubuntu

```sh
sudo apt install ./MusicPlayer_0.1.0_amd64.deb
```

`apt` instala `libmpv2` y `yt-dlp`. Si la versión de `yt-dlp` de tu distribución es antigua y YouTube falla, instala la más reciente con `pipx install yt-dlp`.

### Linux (AppImage)

1. Instala `mpv` (incluye `libmpv`) y `yt-dlp` con el gestor de paquetes de tu distribución.
2. Descarga el `.AppImage`, dale permiso de ejecución y ábrelo:
   ```sh
   chmod +x MusicPlayer_0.1.0_amd64.AppImage
   ./MusicPlayer_0.1.0_amd64.AppImage
   ```

Necesita FUSE; si no lo tienes, ejecútalo con `--appimage-extract-and-run`.

### Arch Linux / Omarchy (recomendado)

Se instala como paquete de `pacman`: sobrevive a las actualizaciones del sistema y aparece en el lanzador.

```sh
sudo pacman -S --needed base-devel git bun rustup
rustup default stable
git clone https://github.com/RicardoMonroy/MusicPlayer.git
cd MusicPlayer/packaging/arch
makepkg -si
```

Instala `musicplayer-desktop` (alias `musicplayer`) y `musicplayer-tui`, con sus entradas en el menú. Para actualizar a una versión nueva, sube `pkgver` en el `PKGBUILD` y repite `makepkg -si`. Para desinstalar: `sudo pacman -R musicplayer`.

### Solo la TUI (Linux)

```sh
tar -xzf musicplayer-tui-v0.1.0-linux-x86_64.tar.gz
install -Dm755 musicplayer-tui-v0.1.0-linux-x86_64/musicplayer-tui ~/.local/bin/musicplayer-tui
```

Requiere `mpv` (libmpv) y `yt-dlp`.

## Primeros pasos

### Música local

- **Escritorio**: en **Biblioteca**, pulsa **Elegir carpeta** y selecciona tu carpeta de música. El primer escaneo de una biblioteca grande tarda un poco; luego las etiquetas quedan en caché.
- **TUI**: usa la misma carpeta que el escritorio. También puedes indicarla al abrirla, y la recuerda: `musicplayer-tui ~/Música`

### Iniciar sesión en YouTube Music

Sin cuenta puedes buscar y reproducir casi todo. Inicia sesión para ver tu biblioteca, tus playlists, "Me gusta" y las canciones que piden sesión.

1. En el navegador, abre una **ventana privada** e inicia sesión en [music.youtube.com](https://music.youtube.com).
2. Instala la extensión **Get cookies.txt LOCALLY** ([Chrome](https://chromewebstore.google.com/detail/get-cookiestxt-locally/cclelndahbckbenkjhflpdbgdldlbecc), [Firefox](https://addons.mozilla.org/firefox/addon/get-cookies-txt-locally/)) y permítele funcionar en ventanas privadas.
3. En la pestaña de music.youtube.com, exporta las cookies como `cookies.txt`.
4. Cierra la ventana privada **sin cerrar sesión**: así esas cookies siguen siendo válidas.
5. En la app de escritorio, ve a **YouTube Music → Conectar cuenta** y elige el `cookies.txt`.

La TUI comparte la sesión del escritorio: no tienes que conectarla aparte. Las cookies se guardan solo en tu equipo. Para salir, usa **Desconectar cuenta**. Si YouTube empieza a fallar con canciones que antes funcionaban, las cookies expiraron: repite los pasos.

### Atajos de teclado

En el escritorio, pulsa `?` para ver todos los atajos. Los más usados:

| Tecla | Acción |
|---|---|
| `Espacio` | Reproducir / pausar |
| `N` / `P` | Siguiente / anterior |
| `←` / `→` | Retroceder / adelantar 5 s |
| `Q` | Cola de reproducción |
| `L` / `T` | Letra / traducción al español |
| `E` | Ecualizador |
| `M` | Mini reproductor |
| `Ctrl + F` | Buscar en la vista actual |

En la TUI pulsa `?` (o `F1`) para la ayuda completa. Los más usados:

| Tecla | Acción |
|---|---|
| `1`–`9`, `0` | Secciones (Local, YouTube Music, Cola; `0` ecualizador) |
| `/` | Buscar |
| `Enter` | Abrir / reproducir |
| `Espacio`, `n` / `p` | Pausar, siguiente / anterior |
| `a` / `d` | Ir al artista / disco de YouTube Music |
| `y` | Copiar la URL del álbum de YouTube Music |
| `l`, `T` | Letra, traducción |
| `t` | Cambiar tema |
| `q` | Salir |

### Copiar la URL de un álbum

En la **Cola** (o en la página de un álbum) de YouTube Music, pulsa **Copiar URL del álbum** (en la TUI, `y`). Copia un enlace `https://music.youtube.com/playlist?list=OLAK5uy_…`, el que usan los descargadores basados en yt-dlp.

## Dónde se guardan tus datos

| Qué | Linux | Windows | macOS |
|---|---|---|---|
| Ajustes (carpeta, skin, ecualizador) | `~/.config/musicplayer/` | `%APPDATA%\musicplayer\` | `~/Library/Application Support/musicplayer/` |
| Sesión de YouTube Music | `~/.local/share/dev.ricardo.musicplayer/ytmusic/` | `%APPDATA%\dev.ricardo.musicplayer\ytmusic\` | `~/Library/Application Support/dev.ricardo.musicplayer/ytmusic/` |
| Caché (biblioteca, portadas, letras) | `~/.cache/musicplayer/` | `%LOCALAPPDATA%\musicplayer\` | `~/Library/Caches/musicplayer/` |

Borrar la caché es seguro: se regenera.

## Problemas comunes

- **YouTube no reproduce nada**: comprueba que `yt-dlp --version` funcione en una terminal y actualízalo; YouTube cambia a menudo.
- **"No se pudo reproducir…; se saltó"**: la canción pide iniciar sesión, o las cookies expiraron.
- **La app no aparece en el menú de Omarchy**: instálala con el `PKGBUILD`. Una instalación manual en `~/.local` depende de tu `PATH`.
- **Cerré la ventana y la música sigue**: la app se minimiza a la bandeja. Usa **Salir** en el ícono de la bandeja para cerrarla.

## Desarrollo

Rust (workspace) + Tauri v2 con SvelteKit para el escritorio, y ratatui para la TUI. La reproducción usa libmpv.

```sh
cd apps/desktop && bun install && bun run tauri dev   # escritorio
cargo run -p musicplayer-tui                          # TUI
cargo test --workspace
```

Plan y decisiones: [docs/PLAN.md](docs/PLAN.md). Cómo publicar una versión: [packaging/README.md](packaging/README.md).

## Licencia

[GPL-3.0-or-later](LICENSE).
