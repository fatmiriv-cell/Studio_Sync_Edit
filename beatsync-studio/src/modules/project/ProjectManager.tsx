import { useEffect, useState } from "react";
import * as api from "../../api/backend";
import { useStore } from "../../state/store";

export default function ProjectManager() {
  const [projects, setProjects] = useState<api.Project[]>([]);
  const [name, setName] = useState("");
  const [error, setError] = useState("");
  const openProject = useStore((s) => s.openProject);

  const refresh = () => api.projectList().then(setProjects).catch((e) => setError(String(e)));
  useEffect(() => {
    refresh();
  }, []);

  const create = async () => {
    if (!name.trim()) return;
    try {
      const p = await api.projectCreate(name.trim());
      await openProject(p);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="pm-shell">
      <div className="pm-card">
        <h1>
          ⚡ AI <span>BeatSync</span> Studio
        </h1>
        <p className="sub">
          Editor AI që sinkronizon çdo prerje me beat-in — 100% offline, lokal, privat.
        </p>

        <div className="pm-row">
          <input
            placeholder="Emri i projektit të ri…"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && create()}
          />
          <button className="primary" onClick={create}>
            Krijo
          </button>
        </div>

        {projects.map((p) => (
          <div key={p.id} className="pm-project" onClick={() => openProject(p)}>
            <div>
              <div>{p.name}</div>
              <div style={{ color: "var(--text-dim)", fontSize: 11 }}>
                {new Date(p.created_at * 1000).toLocaleString()}
              </div>
            </div>
            <button
              className="danger"
              onClick={async (e) => {
                e.stopPropagation();
                await api.projectDelete(p.id);
                refresh();
              }}
            >
              Fshi
            </button>
          </div>
        ))}

        {error && <p style={{ color: "var(--red)", fontSize: 11 }}>{error}</p>}
      </div>
    </div>
  );
}
