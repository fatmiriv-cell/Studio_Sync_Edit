import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import * as api from "../../api/backend";
import { useStore } from "../../state/store";

function fmtDur(s: number) {
  const m = Math.floor(s / 60);
  const sec = Math.floor(s % 60);
  return `${m}:${sec.toString().padStart(2, "0")}`;
}

export default function MediaLibrary() {
  const project = useStore((s) => s.project)!;
  const media = useStore((s) => s.media);
  const refreshMedia = useStore((s) => s.refreshMedia);
  const selectMedia = useStore((s) => s.selectMedia);
  const selectedMediaId = useStore((s) => s.selectedMediaId);
  const videoAnalyses = useStore((s) => s.videoAnalyses);
  const setProgress = useStore((s) => s.setProgress);

  const importFiles = async (kind: "audio" | "video") => {
    const filters =
      kind === "audio"
        ? [{ name: "Muzikë", extensions: ["mp3", "wav", "flac", "m4a", "aac", "ogg"] }]
        : [{ name: "Video", extensions: ["mp4", "mov", "mkv", "avi", "webm", "m4v"] }];
    const picked = await open({ multiple: true, filters });
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    setProgress({ active: true, pct: 50, msg: "Importim…" });
    try {
      await api.mediaImport(project.id, paths as string[]);
      await refreshMedia();
      setProgress({ active: false, pct: 100, msg: "Importi përfundoi" });
    } catch (e) {
      setProgress({ active: false, pct: 0, msg: String(e) });
    }
  };

  return (
    <>
      <div className="panel-header">
        Media Library
        <div style={{ flex: 1 }} />
        <button onClick={() => importFiles("audio")}>+ Muzikë</button>
        <button onClick={() => importFiles("video")}>+ Video</button>
      </div>
      <div className="panel-body">
        {media.length === 0 && (
          <p style={{ color: "var(--text-dim)", fontSize: 12, padding: 8 }}>
            Importo një këngë dhe disa klipe video për të filluar.
          </p>
        )}
        {media.map((m) => {
          const va = videoAnalyses[m.id];
          return (
            <div
              key={m.id}
              className={`media-item ${selectedMediaId === m.id ? "selected" : ""}`}
              onClick={() => selectMedia(m.id)}
            >
              {m.kind === "video" && m.thumbnail ? (
                <img className="media-thumb" src={convertFileSrc(m.thumbnail)} alt="" />
              ) : (
                <div className="media-thumb audio">♪</div>
              )}
              <div className="media-meta">
                <div className="media-name">{m.name}</div>
                <div className="media-sub">
                  {fmtDur(m.duration)}
                  {m.kind === "video" && ` · ${m.width}×${m.height} · ${m.fps.toFixed(0)}fps`}
                </div>
                <div style={{ marginTop: 3 }}>
                  {m.analyzed ? (
                    <span className="badge score">
                      {va ? `AI ${va.quality_score.toFixed(0)}/100` : "Analizuar ✓"}
                    </span>
                  ) : (
                    <span className="badge">Pa analizuar</span>
                  )}
                  {va && va.highlights.length > 0 && (
                    <span className="badge">{va.highlights.length} highlights</span>
                  )}
                </div>
              </div>
              <button
                className="danger"
                style={{ padding: "2px 8px", fontSize: 11 }}
                onClick={async (e) => {
                  e.stopPropagation();
                  await api.mediaRemove(m.id);
                  refreshMedia();
                }}
              >
                ×
              </button>
            </div>
          );
        })}
      </div>
    </>
  );
}
