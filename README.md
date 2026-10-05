# KeepView — Google Keep Desktop Widget (Offline & Auto-Sync)

Aplikasi desktop widget porting untuk **Google Keep** (`https://keep.google.com/u/0/`) yang dibuat menggunakan **Rust**. Memungkinkan Anda menggunakan Google Keep layaknya widget catatan desktop yang tetap bisa diakses saat **offline** dan akan otomatis melakukan **sinkronisasi (sync)** kembali ke cloud saat terhubung ke internet.

---

## 🌟 Fitur Utama

- **📴 Offline-First & Persistent Storage:**
  - Menggunakan profil penyimpanan WebView2 permanen di direktori `%LOCALAPPDATA%\GoogleKeepWidget\WebViewProfile`.
  - Semua catatan, IndexedDB, Service Worker cache, dan sesi login Google tersimpan secara permanen di PC Anda.
  - Saat offline, Anda tetap dapat membaca, membuat catatan baru, mencentang checklist, dan mengedit catatan yang ada.
  - Saat internet terhubung kembali, Google Keep secara otomatis menyinkronkan seluruh perubahan ke akun Google Anda.

- **📱 Mode Widget Desktop Murni (Frameless & Clean):**
  - **No Frame:** Tampilan murni widget tanpa bingkai OS Windows (tanpa titlebar, tombol minimize, maximize/resize, maupun close standar).
  - **No Scroll Slider:** Slider scrollbar disembunyikan sepenuhnya agar tampilan bersih, namun tetap dapat di-scroll lancar dengan mousewheel / touchpad.
  - **Easy Window Drag:** Pindahkan posisi widget dengan drag pada area atas jendela, klik & drag background kosong, atau gunakan ikon `✥` pada floating pill.
  - **Floating Control Pill:** Dilengkapi tombol Pin (`📌`), Geser (`✥`), Pencarian (`🔍`), Zoom, Sinkronisasi (`🔄`), dan Tutup/Sembunyikan ke Tray (`✕`).

- **📌 Always-on-Top (Pin to Desktop):**
  - Kunci widget agar selalu melayang di atas jendela aplikasi lain sehingga mudah mencatat ide kapan saja.
  - Dapat diaktifkan/dinonaktifkan langsung via toolbar widget (`📌`) atau menu System Tray.

- **🔔 Integrasi System Tray (Baki Sistem Windows):**
  - Berjalan hening di latar belakang tanpa memenuhi taskbar.
  - **Klik kiri** ikon baki untuk langsung membuka/menyembunyikan widget.
  - **Klik kanan** untuk menu cepat:
    - Buka / Sembunyikan Widget
    - Toggle Always-on-Top
    - Toggle Mode Kompak
    - Sinkronkan / Muat Ulang
    - Buka Folder Data Offline di File Explorer
    - Keluar

- **⌨️ Global Hotkey (`Alt + Shift + K`):**
  - Tekan `Alt + Shift + K` dari aplikasi apa pun di Windows untuk seketika memunculkan atau menyembunyikan widget Keep.

- **🔗 Safe External Links:**
  - Jika Anda mengklik tautan web (link) yang tercantum di dalam catatan, tautan akan dibuka di browser default Anda (Chrome, Edge, Firefox, dll.), bukan di dalam widget.

- **💾 Window Memory:**
  - Posisi jendela (X, Y), ukuran jendela, status pin, tingkat zoom, dan mode tampilan disimpan secara otomatis di `%APPDATA%\GoogleKeepWidget\config.json`.

- **⚡ Performa Sangat Ringan (Rust Native):**
  - Ukuran file *executable* hanya **~1.4 MB**, konsumsi memori hemat, dan *startup* instan.

---

## 🚀 Cara Menjalankan

### 1. Menjalankan Binary Release
File *executable* hasil kompilasi siap pakai berada di:
```text
f:\Project\keep-view\target\release\keep-view.exe
```
Cukup klik ganda (double click) pada file `keep-view.exe`.

### 2. Menjalankan dari Terminal / Cargo
Untuk pengembangan atau menjalankan lewat cargo:
```bash
cargo run --release
```

---

## 🛠️ Panduan Penggunaan Pertama Kali

1. **Login Akun Google Pertama Kali:**
   - Saat pertama kali dijalankan dalam keadaan online, login ke akun Google Anda.
   - Sesi login dan seluruh catatan akan langsung diunduh dan disimpan ke cache lokal.
2. **Pengujian Mode Offline:**
   - Setelah catatan termuat, Anda bisa mencoba mematikan koneksi internet / Wi-Fi.
   - Indikator status di toolbar widget akan berubah menjadi `📴 Offline (Lokal)`.
   - Buat atau edit catatan baru saat offline.
   - Nyalakan kembali koneksi internet: widget akan mendeteksi koneksi dan status akan berubah menjadi `🔄 Menyinkronkan...` lalu `🟢 Online / Tersinkron`.

---

## 📁 Struktur Penyimpanan Data

- **Konfigurasi Aplikasi:**
  `%APPDATA%\GoogleKeepWidget\config.json`
- **Cache Catatan & Sesi Offline:**
  `%LOCALAPPDATA%\GoogleKeepWidget\WebViewProfile`
  *(Dapat dibuka langsung melalui klik kanan ikon tray -> "Buka Folder Data Offline")*

---

## ⌨️ Daftar Pintasan (Shortcuts)

| Pintasan | Fungsi |
| :--- | :--- |
| `Alt + Shift + K` | Buka / Sembunyikan Widget dari mana saja |
| `Ctrl + R` / `F5` | Muat Ulang & Sinkronkan |
| `Esc` | Tutup jendela bantuan / modal |

---

## 🏗️ Struktur Proyek

```text
keep-view/
├── Cargo.toml              # Dependensi Rust (wry, tao, tray-icon, global-hotkey, dll.)
├── README.md               # Dokumentasi lengkap
└── src/
    ├── main.rs             # Event loop, windowing, IPC bridge, dan offline resilience
    ├── config.rs           # Manajemen konfigurasi persistent
    ├── hotkey.rs           # Registrasi global shortcut Alt+Shift+K
    ├── icon.rs             # Generator ikon Google Keep native 32x32 RGBA
    ├── injected_assets.rs  # Injected widget toolbar, compact CSS, offline sync JS, fallback
    ├── paths.rs            # Manajemen direktori AppData & LocalAppData
    └── tray.rs             # Integrasi system tray icon & menu
```
