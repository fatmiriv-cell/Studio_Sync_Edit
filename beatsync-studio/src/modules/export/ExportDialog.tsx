import { useEffect, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import * as api from "../../api/backend";
import { useStore } from "../../state/store";

const RESOLUTIONS = [
  { label: "1080p (1920×1080)", w: 1920, h: 1080 },
  { label: "1440p (2560×1440)", w: 2560, h: 1440 },
  { label: "4K (3840×2160)", w: 3840, h: 2160 },
  { label: "8K (7680×4320)", w: 7680, h: 4320 },
  { label: "Vertikale 9:16 (1080×1920)", w: 1080, h: 1920 },
];
const FPS = [24, 25, 30, 50, 60];

export default function ExportDialog({ onClose }: { onClose: () => void }) {
  const project = useStore((s) => s.project)!;
  const clips = useStore((s) => s.clips);
  const setProgress = useStore((s) => s.setProgress);
  const [encoders, setEncoders] = useState<string[]>(["libx264"]);
  const [encoder, setEncoder] = useState("libx264");
  const [resIdx, setResIdx] = useState(0);
  const [fps, setFps] = useState(30);
  const [quality, setQuality] = useState(18);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    api.exportEncoders().then((e) => {
      setEncoders(e);
      // Prefero hardware encoder nëse ekziston.
      const hw = e.find((x) => x.includes("nvenc") || x.includes("qsv") || x.includes("amf"));
      if (hw) setEncoder(hw);
    });
  }, []);

  const doExport = async () => {
    const out = await save({
      defaultPath: `${project.name}.mp4`,
      filters: [
        { name: "MP4", extensions: ["mp4"] },
        { name: "MOV", extensions: ["mov"] },
      ],
    });
    if (!out) return;
    setBusy(true);
    setError("");
    try {
      const r = RESOLUTIONS[resIdx];
      await api.exportRender(project.id, {
        output_path: out,
        width: r.w,
        height: r.h,
        fps,
        encoder,
        quality,
      });
      setProgress({ active: false, pct: 100, msg: `U eksportua: ${out}` });
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h3>Eksporto videon</h3>

        <div className="form-row">
          <label>Rezolucioni</label>
          <select value={resIdx} onChange={(e) => setResIdx(Number(e.target.value))}>
            {RESOLUTIONS.map((r, i) => (
              <option key={i} value={i}>{r.label}</option>
            ))}
          </select>
        </div>

        <div className="form-row">
          <label>Frame rate</label>
          <select value={fps} onChange={(e) => setFps(Number(e.target.value))}>
            {FPS.map((f) => (
              <option key={f} value={f}>{f} fps</option>
            ))}
          </select>
        </div>

        <div className="form-row">
          <label>Encoder</label>
          <select value={encoder} onChange={(e) => setEncoder(e.target.value)}>
            {encoders.map((e2) => (
              <option key={e2} value={e2}>
                {e2}
                {e2.includes("nvenc") ? " (NVIDIA GPU)" : ""}
                {e2.includes("qsv") ? " (Intel GPU)" : ""}
                {e2.includes("amf") ? " (AMD GPU)" : ""}
              </option>
            ))}
          </select>
        </div>

        <div className="form-row">
          <label>Cilësia (CRF {quality} — më e ulët = më mirë)</label>
          <input
            type="range"
            min={12}
            max={30}
            value={quality}
            onChange={(e) => setQuality(Number(e.target.value))}
          />
        </div>

        <p style={{ color: "var(--text-dim)", fontSize: 11 }}>
          {clips.length} klipe në timeline · rendering me FFmpeg
          {encoder.includes("264") || encoder.includes("nvenc") ? " · H.264/H.265" : ""}
        </p>

        {error && <p style={{ color: "var(--red)", fontSize: 11 }}>{error}</p>}

        <div className="modal-actions">
          <button onClick={onClose}>Anulo</button>
          <button className="primary" onClick={doExport} disabled={busy || clips.length === 0}>
            {busy ? "Duke renderuar…" : "Eksporto"}
          </button>
        </div>
      </div>
    </div>
  );
}
