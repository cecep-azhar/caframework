# 🏛️ CA Family Design System Specification (CADS v1.0)

> **Document Version:** 1.0.0 (Official Standard)  
> **Author & Architect:** Cecep Saeful Azhar Hidayat, ST (Prof. Cecep)  
> **Target Framework:** CAFramework (`/home/cecepazhar/Product/caframework`)  
> **Applicable Products:** CATerm, CAMark, CATable, CAGames, CABook, CAPost, CAIgent, CAProduct, CAEntech, CAVision, GCC

---

## 1. 🎯 Filosofi & Prinsip Desain

1. **Seamless Dark Shell:** Background terluar (`#0e0e0e`) menyatukan Title Bar dan Sidebar kiri dalam satu kanvas tanpa garis horizontal/divider yang memotong.
2. **Raised Content Card:** Area konten utama (`<main>`) berupa kartu terangkat berlatar `#161616` (Dark) / `#ffffff` (Light) dengan sudut melengkung `rounded-tl-xl` (12px) dan hairline border atas/kiri (`border-neutral-800`).
3. **0px Layout Jump:** Navigasi, expand/collapse sidebar, dan transisi halaman tidak boleh memicu pergeseran posisi vertikal maupun horizontal (tinggi header dikunci `h-14` / 56px, lebar anchor icon 20px).
4. **Non-Destructive Overlays:** Panel pendukung (seperti Hana AI Chat, Notifications, Toasts) berstatus *floating smart card* (`bottom-4 right-4 z-50`) tanpa menekan atau merusak ukuran workspace aktif.
5. **Zero-Knowledge Aesthetics:** Keamanan enkripsi divisualisasikan secara elegan melalui badge status **`Local Identity (Encrypted)`**, live pulsing status dot (`w-1.5 h-1.5` hijau), dan efek visual **Glowing Aurora Spinning Ring** untuk pengguna PRO Founder.

---

## 2. 🎨 Palet Warna Produk (9-Product Brand Matrix)

Setiap aplikasi mengikat satu warna aksen utama yang diturunkan ke seluruh elemen interaktif (`--app-accent`):

| Produk | Nama Warna | Hex Code | Tailwind Primary | CSS Accent Glow |
|---|---|---|---|---|
| **CATerm** | 🔥 Crimson | `#ef4444` | `bg-red-500` | `rgba(239, 68, 68, 0.4)` |
| **CAMark** | 💎 Cyan | `#06b6d4` | `bg-cyan-500` | `rgba(6, 182, 212, 0.4)` |
| **CATable** | 📊 Blue | `#3b82f6` | `bg-blue-500` | `rgba(59, 130, 246, 0.4)` |
| **CAGames** | 🎮 Emerald | `#10b981` | `bg-emerald-500` | `rgba(16, 185, 129, 0.4)` |
| **CABook** | 📚 Amber | `#f59e0b` | `bg-amber-500` | `rgba(245, 158, 11, 0.4)` |
| **CAPost** | 🚀 Indigo | `#6366f1` | `bg-indigo-500` | `rgba(99, 102, 241, 0.4)` |
| **CAIgent** | 🤖 Violet | `#8b5cf6` | `bg-purple-500` | `rgba(139, 92, 246, 0.4)` |
| **CAProduct** | 🌹 Rose | `#f43f5e` | `bg-rose-500` | `rgba(244, 63, 94, 0.4)` |
| **CAFramework** | 🏛️ Platinum | `#64748b` | `bg-slate-500` | `rgba(100, 116, 139, 0.4)` |

---

## 3. 🔘 Standar Desain Tombol (Button Design System)

Semua tombol di seluruh modul dan dialog wajib menggunakan class baku:

### A. Primary Action Button (`variant="primary"`)
Digunakan untuk aksi tambah utama di header dan tombol simpan modal dialog:
```html
<button class="h-9 px-4 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white shadow-md shadow-sky-600/20 active:scale-95 transition-all flex items-center gap-1.5">
  <svg class="w-4 h-4" ... />
  <span>+ Tambah Data</span>
</button>
```

### B. Secondary / Tool Button (`variant="secondary"`)
Digunakan untuk tombol Impor, Ekspor, Filter, dan Aksi Toolbar:
```html
<button class="h-9 px-3.5 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 text-xs font-semibold text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors flex items-center gap-1.5 shadow-2xs">
  <svg class="w-4 h-4 text-neutral-500" ... />
  <span>Ekspor JSON</span>
</button>
```

### C. Card & Row Action Buttons (`variant="row"`)
Digunakan untuk aksi pada kartu atau tabel data (Edit, Hapus, Detail, Copy):
```html
<!-- Edit / View -->
<button class="h-7 px-2.5 rounded-lg text-xs font-medium bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 transition-colors">
  Edit
</button>

<!-- Delete Danger -->
<button class="h-7 px-2.5 rounded-lg text-xs font-medium bg-rose-500/10 hover:bg-rose-600 text-rose-600 dark:text-rose-400 hover:text-white transition-colors">
  Hapus
</button>
```

### D. Icon-Only Action Button (`variant="icon"`)
Digunakan untuk toggle password mata, copy icon, minimize, trigger floating:
```html
<button class="h-9 w-9 flex items-center justify-center rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 text-neutral-500 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors shadow-2xs">
  <svg class="w-4 h-4" ... />
</button>
```

---

## 4. 🪟 Struktur Layout Shell & Metrik Ukuran

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ TITLE BAR (h-[3rem] / 48px · Frameless · Live Pulse Dot · Workspace · QuickControls)   │
├───────────────────┬────────────────────────────────────────────────────────────────────┤
│ SIDEBAR           │ MAIN CONTENT PANEL (`max-w-5xl mx-auto space-y-6`)                 │
│ (w-60 / w-16)     │ ┌────────────────────────────────────────────────────────────────┐ │
│ • Header (h-14)   │ │ PAGE HEADER (Icon Tile 40x40px · Title text-xl · Subtitle)     │ │
│ • Nav Links       │ ├────────────────────────────────────────────────────────────────┤ │
│ • Profile Footer  │ │ CONTENT GRID (Cards 2/3 cols · Filters · Search · Pagination)  │ │
│                   │ └────────────────────────────────────────────────────────────────┘ │
└───────────────────┴────────────────────────────────────────────────────────────────────┘
```

1. **Title Bar Height:** `h-[3rem]` (48px) + `safe-area-inset-top`.
2. **Page Spacing:** Container selalu dibungkus `max-w-5xl mx-auto space-y-6`.
3. **Inner Padding:** `px-4 py-6 md:px-10 md:py-10`.
4. **Header Tile:** Icon tile kotak `w-10 h-10 rounded-xl border flex items-center justify-center`.
5. **Pagination Standard:** Paginasi di bawah tabel/grid dengan pilihan `10 / 25 / 50` item.

---

## 5. 🌸 Spesifikasi Floating Smart Card AI Chat

1. **Posisi Default:** `fixed bottom-4 right-4 z-50 w-[390px] h-[520px] max-h-[85vh]` (non-modal, tidak menutupi workspace).
2. **Minimize Pill:** `_` menyusutkan panel menjadi pill mengambang `Asisten AI Aktif` dengan dot hijau berdenyut.
3. **Header Controls:**
   - Ikon Sparkle Prompt Studio (`M9.813 15.904L9...`).
   - Tombol 🗑️ **Clear Chat** dengan modal konfirmasi.
   - Tombol ↕️ **Expand / Full-Height Mode** (`inset-y-3 right-3 w-[420px]`).
   - Tombol ✕ **Close** (juga tertutup saat menekan tombol `Escape`).
4. **Error Handling:** Jika AI berstatus disabled / unconfigured, munculkan kartu visual langsung dengan tombol menuju `/settings?tab=ai`.

---

## 6. 🛡️ Spesifikasi Lock Screen & Identitas PRO

1. **Split Screen:**
   - Kiri (2/3 lebar): Brand Quote interaktif bergaya dark terminal.
   - Kanan (1/3 lebar): Master Password input form dengan toggle icon mata (show/hide).
2. **Avatar & Badge:**
   - Avatar besar di tengah dengan **Glowing Aurora Spinning Ring** untuk user PRO.
   - Badge Emas bertuliskan **`PRO`** di samping nama.
   - Subtitle: **`Founder Lifetime Edition · Local Identity Encrypted`**.

---

*Setiap aplikasi baru yang dibangun dengan CAFramework wajib mengikuti 100% spesifikasi ini untuk menjamin konsistensi ekosistem CA Family.*
