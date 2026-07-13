// Paneli AI — rrjedha e punës me tre hapa:
// 1. Analizo gjithçka (muzikë + video)  2. Zgjidh stilin  3. Gjenero timeline-in.

import { useEffect, useState } from "react";
import * as api from "../../api/backend";
import { useStore } from "../../state/store";

export default function AiPanel() {
  const project = useStore((s) => s.project)!;
  const media = useStore((s) => s.media);
  const refreshMedia = useStore((s) => s.refreshMedia);
  const audioAnalysis = useStore((s) => s.audioAnalysis);
  const setAudioAnalysis = useStore((s) => s.setAudioAnalysis);
  const setVideoAnalysis = useStore((s) => s.setVideoAnalysis);
  const setClips = useStore((s) => s.setClips);
  const setProgress = useStore((s) => s.setProgress);

  const [styles, setStyles] = useState<api.EditStyle[]>([]);
  const [styleId, setStyleId] = useState("music_video");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    api.autoeditStyles().then(setStyles).catch(() => {});
  }, []);

  const analyzeAll = async () => {
    setBusy(true);
    setError("");
    try {
      for (const m of media) {
        if (m.analyzed) continue;
        if (m.kind === "audio") {
          const a = await api.analyzeAudio(m.id);
          setAudioAnalysis(a);
        } else {
          const v = await api.analyzeVideo(m.id);
          setVideoAnalysis(m.id, v);
        }
      }
      await refreshMedia();
      setProgress({ active: false, pct: 100, msg: "Analiza AI përfundoi ✓" });
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const runAutoEdit = async () => {
    setBusy(true);
    setError("");
    setProgress({ active: true, pct: 50, msg: "AI po ndërton timeline-in…" });
    try {
      const clips = await api.autoeditRun(project.id, styleId);
      setClips(clips);
      setProgress({
        active: false,
        pct: 100,
        msg: `Timeline i gjeneruar: ${clips.length} klipe sinkron me beat ✓`,
      });
    } catch (e) {
      setError(String(e));
      setProgress({ active: false, pct: 0, msg: "" });
    } finally {
      setBusy(false);
    }
  };

  const pending = media.filter((m) => !m.analyzed).length;
  const hasAudio = media.some((m) => m.kind === "audio");
  const hasVideo = media.some((m) => m.kind === "video");

  return (
    <>
      <div className="panel-header">Motori AI</div>
      <div className="panel-body">
        <div className="ai-step">
          <h4>1 · Analiza AI</h4>
          <p>
            BPM, beat map, seksione, energji + skena, lëvizje, cilësi dhe highlights
            për çdo klip. Gjithçka lokalisht.
          </p>
          <button className="primary" onClick={analyzeAll} disabled={busy || pending === 0}>
            {pending === 0 ? "Gjithçka e analizuar ✓" : `Analizo ${pending} skedarë`}
          </button>
          {audioAnalysis && (
            <div style={{ marginTop: 8, fontSize: 11, color: "var(--text-dim)" }}>
              <span className="badge score">{audioAnalysis.bpm.toFixed(1)} BPM</span>
              <span className="badge">{audioAnalysis.beats.length} beats</span>
              <span className="badge">{audioAnalysis.sections.length} seksione</span>
            </div>
          )}
        </div>

        <div className="ai-step">
          <h4>2 · Stili i editimit</h4>
          <div className="style-grid">
            {styles.map((st) => (
              <div
                key={st.id}
                className={`style-card ${styleId === st.id ? "active" : ""}`}
                onClick={() => setStyleId(st.id)}
              >
                <div>{st.name}</div>
                <div className="desc">{st.description}</div>
              </div>
            ))}
          </div>
        </div>

        <div className="ai-step">
          <h4>3 · Auto-Edit</h4>
          <p>
            AI zgjedh highlight-et më të mira dhe pret çdo klip saktësisht mbi beat,
            sipas energjisë së këngës. Timeline-i mbetet plotësisht i editueshëm.
          </p>
          <button
            className="primary"
            onClick={runAutoEdit}
            disabled={busy || !hasAudio || !hasVideo || pending > 0}
          >
            ⚡ Gjenero editimin
          </button>
        </div>

        {error && (
          <p style={{ color: "var(--red)", fontSize: 11, whiteSpace: "pre-wrap" }}>{error}</p>
        )}
      </div>
    </>
  );
}
