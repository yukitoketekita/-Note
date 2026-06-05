pkgname=note-app
pkgver=0.1.0
pkgrel=1
pkgdesc="轻量、本地、随叫随到的便签工具，基于 Tauri 2 + React 构建"
arch=('x86_64')
url="https://github.com/TsukiraiSaigiaochi/-Note"
license=('MIT')
depends=('webkit2gtk-4.1' 'gtk3' 'libayatana-appindicator' 'gcc-libs' 'glib2')
makedepends=('cargo' 'nodejs')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/APP.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/-Note-APP"
    sed -i 's/"identifier": *"[^"]*"/"identifier": "com.note.ipad"/' src-tauri/tauri.conf.json
    npm install -g pnpm --prefix="$srcdir/.pnpm"
    export PATH="$srcdir/.pnpm/bin:$PATH"
    pnpm install --frozen-lockfile
    pnpm tauri build --no-bundle
}

package() {
    cd "$srcdir/-Note-APP"
    install -Dm755 "src-tauri/target/release/ipad" "$pkgdir/usr/bin/note-app"
    install -Dm644 "src-tauri/icons/icon.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/note-app.png"
    mkdir -p "$pkgdir/usr/share/applications"
    cat > "$pkgdir/usr/share/applications/note-app.desktop" << DESKTOP
[Desktop Entry]
Name=ヰnote
Comment=轻量、本地、随叫随到的便签工具
Exec=note-app
Icon=note-app
Type=Application
Categories=Utility;TextEditor;
DESKTOP
}
