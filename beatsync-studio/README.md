# ⚡ AI BeatSync Studio

Editor profesional desktop për Windows që përdor **AI lokale** për të montuar
video automatikisht në sinkron të përsosur me muzikën.

- 100% offline — asnjë skedar nuk largohet nga kompjuteri
- Pa cloud, pa telemetri, pa ngarkime
- GPU acceleration për eksport (NVENC / QSV / AMF)
- Editim jo-destruktiv — timeline plotësisht i editueshëm pas AI-së

## Si funksionon

1. Krijo një projekt
2. Importo një këngë + disa klipe video
3. Kliko **Analizo** — AI llogarit BPM, beat map, seksionet e këngës (verse/chorus/drop),
   dhe për çdo video: skenat, lëvizjen, cilësinë dhe highlight-et
4. Zgjidh një stil (Music Video, Cinematic, Trailer, Wedding, TikTok, …)
5. Kliko **⚡ Gjenero editimin** — AI ndërton timeline-in me çdo prerje mbi beat
6. Rregullo manualisht çfarë të duash (drag, snap-to-beat, delete, undo/redo)
7. **Eksporto** — deri në 8K, 24–60 fps, me encoder hardware

## Si ta marrësh installer-in (.exe)

### Mënyra A — GitHub Actions (pa instaluar asgjë)

1. Krijo një repository të ri në GitHub dhe ngarko këtë projekt:
   ```
   git init && git add -A && git commit -m "initial"
   git branch -M main
   git remote add origin https://github.com/USERNAME/beatsync-studio.git
   git push -u origin main
   ```
2. Hap tab-in **Actions** në GitHub — workflow-i "Build Windows Installer"
   (`.github/workflows/build.yml`) niset vetë: instalon Node 20 + Rust,
   shkarkon FFmpeg dhe e paketon me instaluesin NSIS.
3. Kur të përfundojë (~10–15 min), kliko run-in e fundit dhe shkarko artifact-in
   **AI_BeatSync_Studio_x64-setup** — brenda është `AI_BeatSync_Studio_x64-setup.exe`.
4. Instalo dhe përdor. FFmpeg vjen i përfshirë — s'ka nevojë për asgjë tjetër.

### Mënyra B — Build lokal në Windows

Kërkesat (një herë të vetme):
1. [Node.js 20+](https://nodejs.org)
2. [Rust](https://rustup.rs) (`rustup` — zgjidh default)
3. Microsoft Visual Studio C++ Build Tools (rustup ta kërkon vetë)
4. [FFmpeg](https://www.gyan.dev/ffmpeg/builds/) — kopjo `ffmpeg.exe` + `ffprobe.exe`
   në `src-tauri/bin/` (ose sigurohu që janë në PATH)

Pastaj:
```
npm install
npx tauri build
```
Installer-i del në `src-tauri/target/release/bundle/nsis/*.exe`.

Për zhvillim (hot reload):
```
npm install
npx tauri dev
```

## Arkitektura

Shih [ARCHITECTURE.md](ARCHITECTURE.md) — modulet, kontrata IPC, algoritmet e AI
(spectral flux, beat tracking me programim dinamik, zbulimi i highlight-eve),
dhe si të shtohen modele ONNX (fytyra/emocione/objekte) në `src-tauri/src/ai/`.

## Privatësia

E gjithë analiza (muzikë + video) dhe rendering-u ekzekutohen lokalisht me
FFmpeg + DSP në Rust. Aplikacioni nuk hap asnjë lidhje interneti.
