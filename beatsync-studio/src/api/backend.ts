// Wrapper-a të tipizuar mbi komandat Tauri (IPC).
// I vetmi vend ku frontend-i flet me backend-in.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Project {
  id: string;
  name: string;
  style: string;
  created_at: number;
}

export interface MediaItem {
  id: string;
  project_id: string;
  kind: "audio" | "video";
  path: string;
  name: string;
  duration: number;
  width: number;
  height: number;
  fps: number;
  thumbnail: string | null;
  analyzed: boolean;
}

export interface Section {
  label: string;
  start: number;
  end: number;
  energy: number;
}

export interface AudioAnalysis {
  duration: number;
  bpm: number;
  beats: number[];
  downbeats: number[];
  energy_per_beat: number[];
  sections: Section[];
  accent_beats: number[];
}

export interface Highlight {
  start: number;
  end: number;
  score: number;
}

export interface VideoAnalysis {
  duration: number;
  scenes: number[];
  motion: number[];
  analysis_fps: number;
  avg_brightness: number;
  avg_sharpness: number;
  shake: number;
  quality_score: number;
  highlights: Highlight[];
}

export interface EditStyle {
  id: string;
  name: string;
  description: string;
  beats_low: number;
  beats_high: number;
  prefer_downbeats: boolean;
  transition: string;
  speed_ramps: boolean;
  motion_bias: number;
  aspect: string;
}

export interface ClipRow {
  id: string;
  project_id: string;
  track: number;
  media_id: string;
  timeline_start: number;
  in_point: number;
  out_point: number;
  speed: number;
  transition: string;
}

export interface ExportSettings {
  output_path: string;
  width: number;
  height: number;
  fps: number;
  encoder: string;
  quality: number;
}

export interface ProgressEvent {
  job: "audio" | "video" | "export";
  target_id: string;
  pct: number;
  msg: string;
}

// ---- Projektet ----
export const projectCreate = (name: string) =>
  invoke<Project>("project_create", { name });
export const projectList = () => invoke<Project[]>("project_list");
export const projectDelete = (id: string) => invoke<void>("project_delete", { id });

// ---- Media ----
export const mediaImport = (projectId: string, paths: string[]) =>
  invoke<MediaItem[]>("media_import", { projectId, paths });
export const mediaList = (projectId: string) =>
  invoke<MediaItem[]>("media_list", { projectId });
export const mediaRemove = (mediaId: string) =>
  invoke<void>("media_remove", { mediaId });
export const mediaWaveform = (mediaId: string, bins: number) =>
  invoke<number[]>("media_waveform", { mediaId, bins });

// ---- Analiza AI ----
export const analyzeAudio = (mediaId: string) =>
  invoke<AudioAnalysis>("analyze_audio", { mediaId });
export const analyzeVideo = (mediaId: string) =>
  invoke<VideoAnalysis>("analyze_video", { mediaId });
export const analysisGet = (mediaId: string, kind: "audio" | "video") =>
  invoke<string | null>("analysis_get", { mediaId, kind });

// ---- Auto-Edit ----
export const autoeditStyles = () => invoke<EditStyle[]>("autoedit_styles");
export const autoeditRun = (projectId: string, styleId: string) =>
  invoke<ClipRow[]>("autoedit_run", { projectId, styleId });

// ---- Timeline ----
export const timelineGet = (projectId: string) =>
  invoke<ClipRow[]>("timeline_get", { projectId });
export const timelineUpdateClip = (clip: ClipRow) =>
  invoke<void>("timeline_update_clip", { clip });
export const timelineDeleteClip = (clipId: string) =>
  invoke<void>("timeline_delete_clip", { clipId });
export const timelineAddClip = (clip: ClipRow) =>
  invoke<ClipRow>("timeline_add_clip", { clip });

// ---- Eksporti ----
export const exportEncoders = () => invoke<string[]>("export_encoders");
export const exportRender = (projectId: string, settings: ExportSettings) =>
  invoke<string>("export_render", { projectId, settings });

// ---- Event-et e progresit ----
export const onProgress = (
  channel: "analysis://progress" | "export://progress",
  cb: (e: ProgressEvent) => void,
): Promise<UnlistenFn> => listen<ProgressEvent>(channel, (ev) => cb(ev.payload));
