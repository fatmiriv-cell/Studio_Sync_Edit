// Store global (Zustand): projekt aktiv, media, analiza, timeline, undo/redo.

import { create } from "zustand";
import type {
  AudioAnalysis,
  ClipRow,
  MediaItem,
  Project,
  VideoAnalysis,
} from "../api/backend";
import * as api from "../api/backend";

interface ProgressState {
  active: boolean;
  pct: number;
  msg: string;
}

interface StoreState {
  project: Project | null;
  media: MediaItem[];
  audioAnalysis: AudioAnalysis | null;
  videoAnalyses: Record<string, VideoAnalysis>;
  clips: ClipRow[];
  selectedClipId: string | null;
  selectedMediaId: string | null;
  playhead: number;
  zoom: number; // px për sekondë
  progress: ProgressState;
  // undo/redo — fotografi të timeline-it
  past: ClipRow[][];
  future: ClipRow[][];

  openProject: (p: Project) => Promise<void>;
  closeProject: () => void;
  refreshMedia: () => Promise<void>;
  setClips: (clips: ClipRow[], recordHistory?: boolean) => void;
  updateClip: (clip: ClipRow) => Promise<void>;
  deleteClip: (clipId: string) => Promise<void>;
  undo: () => void;
  redo: () => void;
  setPlayhead: (t: number) => void;
  setZoom: (z: number) => void;
  select: (clipId: string | null) => void;
  selectMedia: (mediaId: string | null) => void;
  setProgress: (p: ProgressState) => void;
  setAudioAnalysis: (a: AudioAnalysis | null) => void;
  setVideoAnalysis: (mediaId: string, a: VideoAnalysis) => void;
}

export const useStore = create<StoreState>((set, get) => ({
  project: null,
  media: [],
  audioAnalysis: null,
  videoAnalyses: {},
  clips: [],
  selectedClipId: null,
  selectedMediaId: null,
  playhead: 0,
  zoom: 40,
  progress: { active: false, pct: 0, msg: "" },
  past: [],
  future: [],

  openProject: async (p) => {
    set({
      project: p,
      media: [],
      clips: [],
      audioAnalysis: null,
      videoAnalyses: {},
      past: [],
      future: [],
      playhead: 0,
    });
    const media = await api.mediaList(p.id);
    const clips = await api.timelineGet(p.id);
    set({ media, clips });
    // Ringarko analizat e ruajtura.
    for (const m of media) {
      if (!m.analyzed) continue;
      if (m.kind === "audio") {
        const j = await api.analysisGet(m.id, "audio");
        if (j) set({ audioAnalysis: JSON.parse(j) });
      } else {
        const j = await api.analysisGet(m.id, "video");
        if (j)
          set((s) => ({
            videoAnalyses: { ...s.videoAnalyses, [m.id]: JSON.parse(j) },
          }));
      }
    }
  },

  closeProject: () => set({ project: null }),

  refreshMedia: async () => {
    const p = get().project;
    if (!p) return;
    set({ media: await api.mediaList(p.id) });
  },

  setClips: (clips, recordHistory = true) =>
    set((s) => ({
      clips,
      past: recordHistory ? [...s.past.slice(-49), s.clips] : s.past,
      future: recordHistory ? [] : s.future,
    })),

  updateClip: async (clip) => {
    const s = get();
    s.setClips(s.clips.map((c) => (c.id === clip.id ? clip : c)));
    await api.timelineUpdateClip(clip);
  },

  deleteClip: async (clipId) => {
    const s = get();
    s.setClips(s.clips.filter((c) => c.id !== clipId));
    set({ selectedClipId: null });
    await api.timelineDeleteClip(clipId);
  },

  undo: () => {
    const { past, clips, future } = get();
    if (past.length === 0) return;
    const prev = past[past.length - 1];
    set({ clips: prev, past: past.slice(0, -1), future: [clips, ...future] });
    prev.forEach((c) => api.timelineUpdateClip(c).catch(() => {}));
  },

  redo: () => {
    const { past, clips, future } = get();
    if (future.length === 0) return;
    const next = future[0];
    set({ clips: next, past: [...past, clips], future: future.slice(1) });
    next.forEach((c) => api.timelineUpdateClip(c).catch(() => {}));
  },

  setPlayhead: (t) => set({ playhead: Math.max(0, t) }),
  setZoom: (z) => set({ zoom: Math.min(400, Math.max(5, z)) }),
  select: (clipId) => set({ selectedClipId: clipId }),
  selectMedia: (mediaId) => set({ selectedMediaId: mediaId }),
  setProgress: (progress) => set({ progress }),
  setAudioAnalysis: (audioAnalysis) => set({ audioAnalysis }),
  setVideoAnalysis: (mediaId, a) =>
    set((s) => ({ videoAnalyses: { ...s.videoAnalyses, [mediaId]: a } })),
}));
