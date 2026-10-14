import { useEffect, useRef } from "react";
import type { SceneId } from "./catalog";
import "./scene.css";

/** 全屏背景和氛围面板里的预览用同一张画面。 */
export function SceneBackdrop({ scene }: { scene: SceneId }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas || (scene !== "rain" && scene !== "stars")) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    let frame = 0;
    let alive = true;
    const draw = () => {
      if (!alive) return;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight;
      if (canvas.width !== w || canvas.height !== h) {
        canvas.width = w;
        canvas.height = h;
      }
      ctx.clearRect(0, 0, w, h);
      if (scene === "rain") {
        ctx.strokeStyle = "rgba(210, 220, 235, 0.35)";
        ctx.lineWidth = 1;
        for (let i = 0; i < 48; i++) {
          const x = ((i * 97 + frame) % (w + 40)) - 20;
          const y = ((i * 53 + frame * 3) % (h + 40)) - 20;
          ctx.beginPath();
          ctx.moveTo(x, y);
          ctx.lineTo(x - 6, y + 14);
          ctx.stroke();
        }
      } else {
        for (let i = 0; i < 36; i++) {
          const x = (i * 131) % w;
          const y = (i * 77) % h;
          const twinkle = 0.35 + 0.65 * Math.abs(Math.sin(frame / 40 + i));
          ctx.fillStyle = `rgba(230, 236, 245, ${twinkle})`;
          ctx.fillRect(x, y, 1.5, 1.5);
        }
      }
      frame += 1;
      requestAnimationFrame(draw);
    };
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduce) {
      frame = 10;
      draw();
      alive = false;
    } else {
      draw();
    }
    return () => {
      alive = false;
    };
  }, [scene]);

  return (
    <div className={`scene-bg scene-${scene}`}>
      {(scene === "rain" || scene === "stars") && <canvas ref={ref} />}
    </div>
  );
}
