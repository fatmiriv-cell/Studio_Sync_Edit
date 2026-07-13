# AI BeatSync Studio — Arkitektura

Editor profesional desktop për Windows që përdor AI **lokale** për të montuar video në sinkron të përsosur me muzikën. 100% offline, pa cloud, pa telemetri.

## Stack-u

| Shtresa | Teknologjia |
|---|---|
| Frontend | React 18 + TypeScript + Vite |
| Desktop Runtime | Tauri 2 |
| Backend | Rust |
| Video Processing | FFmpeg (i bashkangjitur si sidecar) |
| AI Runtime | ONNX Runtime (moduli `ai/`, i gatshëm për modele) |
| Databaza | SQLite (rusqlite, bundled) |
| GPU | NVENC / QSV / AMF për encoding; DirectML/CUDA për ONNX (e ardhmja) |

## Parimet

- **100% lokale** — asnjë skedar i përdoruesit nuk largohet nga kompjuteri.
- **Jo-destruktive** — media origjinale nuk preket kurrë; timeline-i është vetëm referenca + in/out points në DB.
- **Modulare** — çdo modul (audio, video, autoedit, export, ai) është një krate/dosje e izoluar me API të pastër.
- **UI e ndarë nga procesimi** — frontend-i komunikon me backend-in vetëm përmes komandave Tauri (IPC) + events për progres.

## Struktura e projektit

```
beatsync-studio/
├── src/                        # Frontend (React + TS)
│   ├── api/                    # Wrapper-a të tipizuar mbi Tauri invoke
│   ├── state/                  # Zustand store (projekt, media, timeline)
│   ├── modules/
│   │   ├── project/            # Project Manager (krijo/hap projekte)
│   │   ├── media/              # Media Library (import, thumbnails, score)
│   │   ├── preview/            # Preview Window (video player)
│   │   ├── timeline/           # Timeline profesional (tracks, clips, beats)
│   │   ├── export/             # Dialog eksporti
│   │   └── settings/           # Cilësimet (GPU, cache, stil)
│   └── styles/                 # Tema e errët profesionale
├── src-tauri/                  # Backend (Rust)
│   └── src/
│       ├── commands/           # Komandat IPC (API publike e backend-it)
│       ├── db/                 # SQLite: schema + modele
│       ├── media/              # FFmpeg wrapper, probe, thumbnails, waveform
│       ├── audio/              # MOTORI AI I MUZIKËS (DSP në Rust)
│       │   ├── decode.rs       # Dekodim → PCM mono f32
│       │   ├── onset.rs        # STFT + spectral flux (onset envelope)
│       │   ├── tempo.rs        # BPM (autokorrelacion) + beat tracking (DP, metoda Ellis)
│       │   └── structure.rs    # Energji, seksione (intro/verse/chorus/drop/outro), downbeats
│       ├── video/              # MOTORI AI I VIDEOS
│       │   ├── scenes.rs       # Kufijtë e skenave (FFmpeg scdet)
│       │   ├── motion.rs       # Intensiteti i lëvizjes, shkëlqimi, mprehtësia
│       │   └── highlights.rs   # Score cilësie + nxjerrja e highlight-eve
│       ├── autoedit/           # MOTORI AUTO-EDIT
│       │   ├── styles.rs       # Presetet (Music Video, Cinematic, TikTok, …)
│       │   └── engine.rs       # Gjenerimi i cut-list sinkron me beat
│       ├── export/             # Render me FFmpeg, hardware encoding
│       └── ai/                 # Shtresa ONNX Runtime (fytyra/emocione — e gatshme për zgjerim)
└── .github/workflows/          # CI: ndërton Windows installer (.exe) automatikisht
```

## Rrjedha e të dhënave (workflow-i i përdoruesit)

1. `project_create` → rresht i ri në SQLite, dosje cache.
2. `media_import(paths)` → FFmpeg probe (kohëzgjatja, rezolucioni, fps) + thumbnail → tabela `media`.
3. `analyze_audio(media_id)` → PCM → onset envelope → BPM + beats + downbeats + energji + seksione → tabela `audio_analysis` (JSON beat map). Progresi emitohet si event `analysis://progress`.
4. `analyze_video(media_id)` → skena + kurba e lëvizjes + score cilësie + highlights → tabela `video_analysis`.
5. `autoedit_run(style)` → motori lexon beat map + highlights → gjeneron cut-list (çdo prerje bie mbi beat; gjatësia e klipeve sipas energjisë dhe stilit) → shkruhet si `timeline_clips` në DB.
6. UI e shfaq timeline-in — plotësisht i editueshëm (trim, move, delete, snap-to-beat, undo/redo në store).
7. `export_render(settings)` → FFmpeg filter_complex (trim + concat + muzika + fades) me encoder hardware kur mbështetet.

## Kontrata IPC (commands)

Çdo komandë kthen `Result<T, String>`; operacionet e gjata raportojnë progres me `app.emit("<kanal>://progress", {job, pct, msg})`.

| Komanda | Përshkrimi |
|---|---|
| `project_create / project_open / project_list` | Menaxhimi i projekteve |
| `media_import(project_id, paths)` | Import + probe + thumbnail |
| `media_waveform(media_id)` | Peaks të waveform-it për timeline |
| `analyze_audio(media_id)` | Beat map i plotë |
| `analyze_video(media_id)` | Skena, lëvizje, highlights, score |
| `autoedit_run(project_id, style)` | Gjeneron timeline-in |
| `timeline_get / timeline_update_clip / timeline_delete_clip` | Editim jo-destruktiv |
| `export_render(project_id, settings)` | Render final |
| `export_encoders()` | Zbulon encoder-ët hardware në dispozicion |

## AI lokale — si funksionon pa cloud

- **Analiza e muzikës**: DSP e pastër në Rust (rustfft). Spectral flux → onset envelope; autokorrelacion → BPM; programim dinamik (metoda Ellis) → pozicionet e beat-eve; faza me energji maksimale → downbeats; RMS i segmentuar → seksionet e këngës dhe drops.
- **Analiza e videos**: FFmpeg si motor ekstraktimi (scdet për skena, diferencë frame-sh për lëvizje, signalstats për ndriçim/mprehtësi) — pa dekodim të dyfishtë në Rust.
- **Moduli `ai/`**: shtresë e izoluar ONNX Runtime me trait `AiModel`; modelet (fytyra, emocione, objekte) shtohen si skedarë `.onnx` në `models/` pa prekur pjesën tjetër të kodit. CPU fallback gjithmonë; DirectML/CUDA kur ka GPU.

## Zgjerueshmëria

- Module të reja AI = implemento trait-in `AiModel` + regjistro në `ai/registry`.
- Stile të reja editimi = një strukturë `EditStyle` e re në `autoedit/styles.rs`.
- Plugin Manager (e ardhmja): dosja `plugins/` me manifest JSON + dynamic loading.
