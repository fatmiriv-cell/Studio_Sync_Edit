// Preview Window: luan klipin e zgjedhur të timeline-it (me in/out points)
// ose median e zgjedhur nga libraria. Playback i medias lokale përmes asset protocol.

import { useEffect, useMemo, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useStore } from "../../state/store";

function tc(t: number) {
  const m = Math.floor(t / 60);
  const s = Math.floor(t % 60);
  const f = Math.floor((t % 1) * 30);
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}:${f
    .toString()
    .padStart(2, "0")}`;
}

export default function Preview() {
  const media = useStore((s) => s.media);
  const clips = useStore((s) => s.clips);
  const selectedClipId = useStore((s) => s.selectedClipId);
  const selectedMediaId = useStore((s) => s.selectedMediaId);
  const playhead = useStore((s) => s.playhead);
  const setPlayhead = useStore((s) => s.setPlayhead);
  const videoRef = useRef<HTMLVideoElement>(null);
  const [playing, setPlaying] = useState(false);

  const clip = clips.find((c) => c.id === selectedClipId) ?? null;
  const sourceMedia = useMemo(() => {
    if (clip) return media.find((m) => m.id === clip.media_id) ?? null;
    if (selectedMediaId) {
      const m = media.find((x) => x.id === selectedMediaId);
      return m?.kind === "video" ? m : null;
    }
    return null;
  }, [clip, selectedMediaId, media]);

  const src = sourceMedia ? convertFileSrc(sourceMedia.path) : "";

  // Kur zgjidhet një klip timeline-i, kërko te in_point i tij.
  useEffect(() => {
    const v = videoRef.current;
    if (!v || !clip) return;
    const seekTo = clip.in_point;
    const onLoaded = () => {
      v.currentTime = seekTo;
    };
    v.addEventListener("loadedmetadata", onLoaded);
    if (v.readyState >= 1) v.currentTime = seekTo;
    return () => v.removeEventListener("loadedmetadata", onLoaded);
  }, [clip?.id, src]);

  // Ndal playback-un kur kalon out_point (preview i saktë i klipit).
  useEffect(() => {
    const v = videoRef.current;
    if (!v) return;
    const onTime = () => {
      if (clip && v.currentTime >= clip.out_point) {
        v.pause();
        setPlaying(false);
      }
      if (clip) {
        setPlayhead(clip.timeline_start + (v.currentTime - clip.in_point) / clip.speed);
      }
    };
    v.addEventListener("timeupdate", onTime);
    return () => v.removeEventListener("timeupdate", onTime);
  }, [clip, setPlayhead]);

  const toggle = () => {
    const v = videoRef.current;
    if (!v) return;
    if (v.paused) {
      if (clip && v.currentTime >= clip.out_point) v.currentTime = clip.in_point;
      if (clip) v.playbackRate = 1 / clip.speed;
      v.play();
      setPlaying(true);
    } else {
      v.pause();
      setPlaying(false);
    }
  };

  return (
    <>
      <div className="preview-video-wrap">
        {src ? (
          <video ref={videoRef} src={src} onClick={toggle} />
        ) : (
          <div style={{ color: "var(--text-dim)", textAlign: "center" }}>
            <div style={{ fontSize: 40, marginBottom: 10 }}>▶</div>
            Zgjidh një klip nga timeline-i ose një video nga libraria
          </div>
        )}
      </div>
      <div className="preview-controls">
        <button onClick={toggle} disabled={!src}>
          {playing ? "⏸ Pauzë" : "▶ Luaj"}
        </button>
        <span className="timecode">{tc(playhead)}</span>
        {clip && (
          <span style={{ color: "var(--text-dim)", fontSize: 11 }}>
            klip: {tc(clip.in_point)} → {tc(clip.out_point)}
            {clip.speed !== 1 && ` · ${clip.speed}x`}
          </span>
        )}
      </div>
    </>
  );
}
