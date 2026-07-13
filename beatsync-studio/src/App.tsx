import { useEffect, useState } from "react";
import { useStore } from "./state/store";
import { onProgress } from "./api/backend";
import ProjectManager from "./modules/project/ProjectManager";
import MediaLibrary from "./modules/media/MediaLibrary";
import Preview from "./modules/preview/Preview";
import Timeline from "./modules/timeline/Timeline";
import AiPanel from "./modules/media/AiPanel";
import ExportDialog from "./modules/export/ExportDialog";

export default function App() {
  const project = useStore((s) => s.project);
  const progress = useStore((s) => s.progress);
  const setProgress = useStore((s) => s.setProgress);
  const closeProject = useStore((s) => s.closeProject);
  const undo = useStore((s) => s.undo);
  const redo = useStore((s) => s.redo);
  const [showExport, setShowExport] = useState(false);

  // Event-et e progresit nga backend-i.
  useEffect(() => {
    const un1 = onProgress("analysis://progress", (e) =>
      setProgress({ active: e.pct < 100, pct: e.pct, msg: e.msg }),
    );
    const un2 = onProgress("export://progress", (e) =>
      setProgress({ active: e.pct < 100, pct: e.pct, msg: e.msg }),
    );
    return () => {
      un1.then((f) => f());
      un2.then((f) => f());
    };
  }, [setProgress]);

  // Shkurtoret globale të tastierës.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key.toLowerCase() === "z" && !e.shiftKey) {
        e.preventDefault();
        undo();
      } else if (
        (e.ctrlKey && e.key.toLowerCase() === "y") ||
        (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "z")
      ) {
        e.preventDefault();
        redo();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [undo, redo]);

  if (!project) return <ProjectManager />;

  return (
    <div className="app-shell">
      <div className="topbar">
        <span className="logo">⚡ AI BeatSync Studio</span>
        <span style={{ color: "var(--text-dim)" }}>{project.name}</span>
        <div className="spacer" />
        <button onClick={() => setShowExport(true)} className="primary">
          Eksporto
        </button>
        <button onClick={closeProject}>Mbyll projektin</button>
      </div>

      <div className="main-area">
        <div className="panel" style={{ width: 280, flexShrink: 0 }}>
          <MediaLibrary />
        </div>
        <div className="preview-area">
          <Preview />
        </div>
        <div
          className="panel"
          style={{ width: 300, flexShrink: 0, borderLeft: "1px solid var(--border)", borderRight: "none" }}
        >
          <AiPanel />
        </div>
      </div>

      <Timeline />

      <div className="statusbar">
        {progress.active ? (
          <>
            <div className="progress-bar" style={{ width: 160 }}>
              <div style={{ width: `${progress.pct}%` }} />
            </div>
            <span>{progress.msg}</span>
          </>
        ) : (
          <span>{progress.msg || "Gati"}</span>
        )}
        <div className="spacer" style={{ flex: 1 }} />
        <span>100% lokale · offline · pa cloud</span>
      </div>

      {showExport && <ExportDialog onClose={() => setShowExport(false)} />}
    </div>
  );
}
