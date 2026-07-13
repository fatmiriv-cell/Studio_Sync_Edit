// Timeline profesional: ruler me beats/downbeats, seksionet e këngës me ngjyra,
// waveform i muzikës, klipet video me drag + snap-to-beat, zoom, delete, undo/redo.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as api from "../../api/backend";
import { useStore } from "../../state/store";

const SECTION_COLORS: Record<string, string> = {
  intro: "#4a5fc4",
  verse: "#3a7d5c",
  build_up: "#c48f2c",
  chorus: "#c44a8f",
  drop: "#d9ff2e",
  bridge: "#7d5cc4",
  outro: "#5c6b7d",
  song: "#3a4a5c",
};

export default function Timeline() {
  const clips = useStore((s) => s.clips);
  const media = useStore((s) => s.media);
  const audio = useStore((s) => s.audioAnalysis);
  const zoom = useStore((s) => s.zoom);
  const setZoom = useStore((s) => s.setZoom);
  const playhead = useStore((s) => s.playhead);
  const setPlayhead = useStore((s) => s.setPlayhead);
  const selectedClipId = useStore((s) => s.selectedClipId);
  const select = useStore((s) => s.select);
  const updateClip = useStore((s) => s.updateClip);
  const deleteClip = useStore((s) => s.deleteClip);
  const undo = useStore((s) => s.undo);
  const redo = useStore((s) => s.redo);

  const [snap, setSnap] = useState(true);
  const [waveform, setWaveform] = useState<number[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);
  const waveCanvasRef = useRef<HTMLCanvasElement>(null);

  const musicMedia = media.find((m) => m.kind === "audio") ?? null;
  const duration = Math.max(
    audio?.duration ?? 0,
    musicMedia?.duration ?? 0,
    ...clips.map((c) => c.timeline_start + (c.out_point - c.in_point) / c.speed),
    30,
  );
  const width = duration * zoom;

  // Waveform i muzikës.
  useEffect(() => {
    if (!musicMedia) return;
    api.mediaWaveform(musicMedia.id, 2000).then(setWaveform).catch(() => {});
  }, [musicMedia?.id]);

  useEffect(() => {
    const canvas = waveCanvasRef.current;
    if (!canvas || waveform.length === 0) return;
    canvas.width = Math.min(width, 32000);
    canvas.height = 48;
    const ctx = canvas.getContext("2d")!;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#4caf50";
    const bw = canvas.width / waveform.length;
    waveform.forEach((p, i) => {
      const h = Math.max(1, p * 44);
      ctx.fillRect(i * bw, (48 - h) / 2, Math.max(1, bw - 0.5), h);
    });
  }, [waveform, width]);

  // Delete e klipit të zgjedhur.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.key === "Delete" || e.key === "Backspace") && selectedClipId) {
        deleteClip(selectedClipId);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [selectedClipId, deleteClip]);

  const snapTime = useCallback(
    (t: number) => {
      if (!snap || !audio || audio.beats.length === 0) return t;
      let best = audio.beats[0];
      for (const b of audio.beats) {
        if (Math.abs(b - t) < Math.abs(best - t)) best = b;
      }
      // Snap vetëm nëse jemi brenda 0.25s të një beat-i.
      return Math.abs(best - t) < 0.25 ? best : t;
    },
    [snap, audio],
  );

  // Drag i klipeve.
  const dragState = useRef<{ id: string; startX: number; origStart: number } | null>(null);
  const onClipMouseDown = (e: React.MouseEvent, clipId: string) => {
    e.stopPropagation();
    select(clipId);
    const c = clips.find((x) => x.id === clipId);
    if (!c) return;
    dragState.current = { id: clipId, startX: e.clientX, origStart: c.timeline_start };
    const onMove = (ev: MouseEvent) => {
      const d = dragState.current;
      if (!d) return;
      const dt = (ev.clientX - d.startX) / zoom;
      const cl = useStore.getState().clips.find((x) => x.id === d.id);
      if (!cl) return;
      const newStart = snapTime(Math.max(0, d.origStart + dt));
      useStore.setState({
        clips: useStore
          .getState()
          .clips.map((x) => (x.id === d.id ? { ...x, timeline_start: newStart } : x)),
      });
    };
    const onUp = () => {
      const d = dragState.current;
      dragState.current = null;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
      if (!d) return;
      const cl = useStore.getState().clips.find((x) => x.id === d.id);
      if (cl) updateClip(cl);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };

  const onRulerClick = (e: React.MouseEvent) => {
    const rect = scrollRef.current!.getBoundingClientRect();
    const x = e.clientX - rect.left + scrollRef.current!.scrollLeft;
    setPlayhead(x / zoom);
    select(null);
  };

  const beatEls = useMemo(() => {
    if (!audio) return null;
    const downs = new Set(audio.downbeats.map((d) => d.toFixed(2)));
    return audio.beats.map((b, i) => (
      <div
        key={i}
        className={`tl-beat ${downs.has(b.toFixed(2)) ? "downbeat" : ""}`}
        style={{ left: b * zoom }}
      />
    ));
  }, [audio, zoom]);

  return (
    <div className="timeline-wrap">
      <div className="timeline-toolbar">
        <span style={{ fontSize: 11, color: "var(--text-dim)" }}>TIMELINE</span>
        {audio && (
          <span className="badge score">{audio.bpm.toFixed(1)} BPM</span>
        )}
        <div style={{ flex: 1 }} />
        <label style={{ fontSize: 11, color: "var(--text-dim)", cursor: "pointer" }}>
          <input
            type="checkbox"
            checked={snap}
            onChange={(e) => setSnap(e.target.checked)}
            style={{ verticalAlign: "middle", marginRight: 4 }}
          />
          Snap në beat
        </label>
        <button onClick={undo} title="Ctrl+Z">↶</button>
        <button onClick={redo} title="Ctrl+Shift+Z">↷</button>
        <button onClick={() => setZoom(zoom / 1.4)}>−</button>
        <button onClick={() => setZoom(zoom * 1.4)}>+</button>
      </div>

      <div className="timeline-scroll" ref={scrollRef}>
        <div className="timeline-canvas" style={{ width }}>
          {/* Ruler: seksionet + beats */}
          <div className="tl-ruler" onClick={onRulerClick}>
            {audio?.sections.map((s, i) => (
              <div
                key={i}
                className="tl-section"
                title={s.label}
                style={{
                  left: s.start * zoom,
                  width: (s.end - s.start) * zoom,
                  background: SECTION_COLORS[s.label] ?? "#3a4a5c",
                }}
              />
            ))}
            {beatEls}
          </div>

          {/* Track video */}
          <div className="tl-track" onClick={onRulerClick}>
            <div className="tl-track-label">V1 · Video</div>
            {clips.map((c) => {
              const m = media.find((x) => x.id === c.media_id);
              const w = ((c.out_point - c.in_point) / c.speed) * zoom;
              return (
                <div
                  key={c.id}
                  className={`tl-clip ${selectedClipId === c.id ? "selected" : ""}`}
                  style={{ left: c.timeline_start * zoom, width: Math.max(8, w) }}
                  onMouseDown={(e) => onClipMouseDown(e, c.id)}
                  title={`${m?.name ?? ""} · ${c.in_point.toFixed(2)}→${c.out_point.toFixed(2)}`}
                >
                  {c.speed !== 1 && <span className="speed-tag">{c.speed}x </span>}
                  {m?.name ?? c.media_id}
                </div>
              );
            })}
          </div>

          {/* Track audio (muzika + waveform) */}
          <div className="tl-track" onClick={onRulerClick}>
            <div className="tl-track-label">A1 · Muzika</div>
            {musicMedia && (
              <div
                className="tl-audio"
                style={{ left: 0, width: (musicMedia.duration || duration) * zoom }}
              >
                <canvas ref={waveCanvasRef} style={{ width: "100%", height: 48 }} />
              </div>
            )}
          </div>

          <div className="tl-playhead" style={{ left: playhead * zoom }} />
        </div>
      </div>
    </div>
  );
}
