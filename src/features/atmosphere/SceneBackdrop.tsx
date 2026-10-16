import { useEffect, useRef } from "react";
import type { SceneId } from "./catalog";
import "./scene.css";

/** 全屏背景和氛围面板里的预览用同一张画面。内置场景含多层光影景深与细腻粒子动态，导入的照片带自然暗角与柔化。 */
export function SceneBackdrop({ scene, photo }: { scene: SceneId; photo?: string | null }) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas || photo) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let frame = 0;
    let alive = true;
    let rafId = 0;

    const draw = () => {
      if (!alive) return;
      const cssW = canvas.clientWidth || 320;
      const cssH = canvas.clientHeight || 180;
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      const targetW = Math.round(cssW * dpr);
      const targetH = Math.round(cssH * dpr);

      if (canvas.width !== targetW || canvas.height !== targetH) {
        canvas.width = targetW;
        canvas.height = targetH;
      }

      ctx.save();
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, cssW, cssH);

      const t = frame * 0.016;

      if (scene === "rain") {
        drawRainScene(ctx, cssW, cssH, t);
      } else if (scene === "stars") {
        drawStarsScene(ctx, cssW, cssH, t);
      } else if (scene === "lamp") {
        drawLampScene(ctx, cssW, cssH, t);
      } else if (scene === "fog") {
        drawFogScene(ctx, cssW, cssH, t);
      } else {
        drawSkyScene(ctx, cssW, cssH, t);
      }

      ctx.restore();
      frame += 1;
      if (alive) {
        rafId = requestAnimationFrame(draw);
      }
    };

    const reduce =
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduce) {
      frame = 120;
      draw();
      alive = false;
    } else {
      draw();
    }

    return () => {
      alive = false;
      if (rafId) cancelAnimationFrame(rafId);
    };
  }, [scene, photo]);

  if (photo) {
    return (
      <div className="scene-bg scene-photo">
        <img src={photo} alt="" />
        <div className="scene-depth" />
      </div>
    );
  }

  return (
    <div className={`scene-bg scene-${scene}`}>
      <div className="scene-ambient" />
      <canvas ref={ref} />
      <div className="scene-depth" />
    </div>
  );
}

/** 伪随机哈希，保证粒子分布自然无网格感且帧间稳定 */
function hash(n: number): number {
  const x = Math.sin(n * 127.1 + 311.7) * 43758.5453123;
  return x - Math.floor(x);
}

/** 天色：高空柔光呼吸带与极细微的浮游微尘，营造开阔安静的空气感 */
function drawSkyScene(ctx: CanvasRenderingContext2D, w: number, h: number, t: number) {
  // 地平线上方缓慢呼吸的柔光层
  const breath = 0.5 + 0.5 * Math.sin(t * 0.35);
  const glowY = h * (0.78 - 0.03 * breath);
  const grad = ctx.createRadialGradient(w * 0.5, glowY, h * 0.04, w * 0.5, h * 0.85, Math.max(w, h) * 0.65);
  grad.addColorStop(0, `rgba(180, 204, 235, ${0.06 + 0.03 * breath})`);
  grad.addColorStop(0.55, `rgba(145, 175, 215, ${0.025 + 0.015 * breath})`);
  grad.addColorStop(1, "rgba(145, 175, 215, 0)");
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, w, h);

  // 空气中缓慢漂浮的微光粒子
  for (let i = 0; i < 24; i++) {
    const r1 = hash(i * 3 + 1);
    const r2 = hash(i * 3 + 2);
    const r3 = hash(i * 3 + 3);
    const speed = 4 + r3 * 6;
    const x = ((r1 * w + Math.sin(t * 0.4 + i) * 18 + t * speed * 0.35) % (w + 20)) - 10;
    const y = ((r2 * h - t * speed + h * 10) % (h + 20)) - 10;
    const pulse = 0.25 + 0.75 * (0.5 + 0.5 * Math.sin(t * (0.6 + r3 * 0.6) + i * 2.1));
    const alpha = (0.06 + r3 * 0.1) * pulse;
    const radius = 0.8 + r1 * 1.4;

    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fillStyle = `rgba(218, 228, 242, ${alpha})`;
    ctx.fill();
  }
}

/** 雨窗：远景细雨、近景渐变雨丝、窗面散景水珠与窗外路灯柔光折射 */
function drawRainScene(ctx: CanvasRenderingContext2D, w: number, h: number, t: number) {
  // 1. 远景细雨：密集、轻柔、速度略慢，形成窗外雨幕景深
  ctx.lineWidth = 0.75;
  const slantFar = -0.18;
  for (let i = 0; i < 46; i++) {
    const r1 = hash(i * 5 + 1);
    const r2 = hash(i * 5 + 2);
    const r3 = hash(i * 5 + 3);
    const speed = 220 + r3 * 130;
    const len = 12 + r3 * 14;
    const cycleH = h + len * 2 + 40;
    const y = ((r2 * cycleH + t * speed) % cycleH) - len - 20;
    const x = ((r1 * (w + 120) + y * slantFar + w * 4) % (w + 80)) - 40;
    const alpha = 0.07 + r3 * 0.1;

    ctx.strokeStyle = `rgba(192, 212, 236, ${alpha})`;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(x + len * slantFar, y + len);
    ctx.stroke();
  }

  // 2. 近景雨丝：带尾迹渐变的真实运动模糊雨滴
  const slantNear = -0.21;
  for (let i = 0; i < 26; i++) {
    const r1 = hash(i * 7 + 11);
    const r2 = hash(i * 7 + 13);
    const r3 = hash(i * 7 + 17);
    const speed = 420 + r3 * 240;
    const len = 26 + r3 * 28;
    const cycleH = h + len * 2 + 60;
    const y = ((r2 * cycleH + t * speed) % cycleH) - len - 30;
    const x = ((r1 * (w + 160) + y * slantNear + w * 4) % (w + 100)) - 50;
    const ex = x + len * slantNear;
    const ey = y + len;

    const grad = ctx.createLinearGradient(x, y, ex, ey);
    const headAlpha = 0.22 + r3 * 0.24;
    grad.addColorStop(0, "rgba(205, 222, 242, 0)");
    grad.addColorStop(0.7, `rgba(205, 222, 242, ${headAlpha * 0.55})`);
    grad.addColorStop(1, `rgba(226, 238, 252, ${headAlpha})`);

    ctx.lineWidth = 1 + r3 * 0.55;
    ctx.strokeStyle = grad;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(ex, ey);
    ctx.stroke();
  }

  // 3. 窗玻璃上的静置/微滑水珠与散景光斑
  for (let i = 0; i < 18; i++) {
    const r1 = hash(i * 11 + 31);
    const r2 = hash(i * 11 + 37);
    const r3 = hash(i * 11 + 41);
    const drift = (t * (1.2 + r3 * 2.4) + r2 * h) % (h + 24);
    const x = r1 * w + Math.sin(drift * 0.08 + i) * 2.5;
    const y = drift - 12;
    const radius = 1.6 + r3 * 2.8;
    const shimmer = 0.65 + 0.35 * Math.sin(t * 1.4 + i * 1.7);

    // 柔和的水珠折射晕圈
    const dropGrad = ctx.createRadialGradient(
      x - radius * 0.25,
      y - radius * 0.3,
      radius * 0.1,
      x,
      y,
      radius * 2.2,
    );
    const warm = i % 4 === 0;
    const color = warm ? "238, 206, 164" : "202, 222, 245";
    dropGrad.addColorStop(0, `rgba(245, 250, 255, ${0.28 * shimmer})`);
    dropGrad.addColorStop(0.45, `rgba(${color}, ${0.12 * shimmer})`);
    dropGrad.addColorStop(1, `rgba(${color}, 0)`);

    ctx.fillStyle = dropGrad;
    ctx.beginPath();
    ctx.arc(x, y, radius * 2.2, 0, Math.PI * 2);
    ctx.fill();
  }
}

/** 台灯：暖光锥呼吸感与在光束中浮沉、进出光区明暗变化的金色微尘 */
function drawLampScene(ctx: CanvasRenderingContext2D, w: number, h: number, t: number) {
  const lampX = w * 0.72;
  const lampY = h * 0.14;
  const breath = 0.5 + 0.5 * Math.sin(t * 0.55);

  // 台灯暖光核心柔晕
  const coneGrad = ctx.createRadialGradient(
    lampX,
    lampY,
    Math.min(w, h) * 0.02,
    w * 0.58,
    h * 0.48,
    Math.max(w, h) * 0.72,
  );
  coneGrad.addColorStop(0, `rgba(255, 228, 178, ${0.16 + 0.04 * breath})`);
  coneGrad.addColorStop(0.38, `rgba(235, 186, 120, ${0.08 + 0.02 * breath})`);
  coneGrad.addColorStop(0.75, "rgba(196, 140, 82, 0.02)");
  coneGrad.addColorStop(1, "rgba(196, 140, 82, 0)");
  ctx.fillStyle = coneGrad;
  ctx.fillRect(0, 0, w, h);

  // 光束中的浮尘：越靠近灯源至桌面光束轴线越明亮
  for (let i = 0; i < 38; i++) {
    const r1 = hash(i * 13 + 5);
    const r2 = hash(i * 13 + 7);
    const r3 = hash(i * 13 + 9);

    const baseX = (r1 * w + Math.sin(t * (0.25 + r3 * 0.25) + i * 1.9) * 26 + t * (2.5 + r2 * 3)) % w;
    const baseY = ((r2 * h - t * (3.2 + r3 * 4.5) + Math.cos(t * 0.3 + i) * 14) % h + h) % h;

    // 计算粒子到光束中心区域 (从右上 0.72,0.14 投向中下 0.48,0.82) 的距离衰减
    const progressY = Math.max(0, Math.min(1, (baseY - lampY) / (h * 0.78)));
    const beamCenterX = lampX + (w * 0.46 - lampX) * progressY;
    const beamHalfWidth = w * (0.14 + progressY * 0.34);
    const distRatio = Math.abs(baseX - beamCenterX) / Math.max(1, beamHalfWidth);
    const inBeam = Math.max(0.08, Math.exp(-distRatio * distRatio * 1.65));

    const twinkle = 0.55 + 0.45 * Math.sin(t * (0.9 + r3 * 1.1) + i * 2.7);
    const alpha = (0.1 + r3 * 0.34) * inBeam * twinkle;
    const radius = 0.9 + r1 * 2.1;

    const moteGrad = ctx.createRadialGradient(baseX, baseY, 0, baseX, baseY, radius * 2.6);
    moteGrad.addColorStop(0, `rgba(255, 238, 202, ${Math.min(0.75, alpha * 1.5)})`);
    moteGrad.addColorStop(0.45, `rgba(238, 196, 136, ${alpha * 0.7})`);
    moteGrad.addColorStop(1, "rgba(238, 196, 136, 0)");

    ctx.fillStyle = moteGrad;
    ctx.beginPath();
    ctx.arc(baseX, baseY, radius * 2.6, 0, Math.PI * 2);
    ctx.fill();
  }
}

/** 星夜：三层景深星群（冷暖色温、星芒晕圈）、银河柔雾与偶发流星 */
function drawStarsScene(ctx: CanvasRenderingContext2D, w: number, h: number, t: number) {
  // 银河柔光斜带
  ctx.save();
  ctx.translate(w * 0.5, h * 0.42);
  ctx.rotate(-0.38);
  const mwGrad = ctx.createRadialGradient(0, 0, h * 0.02, 0, 0, Math.max(w, h) * 0.62);
  mwGrad.addColorStop(0, "rgba(155, 182, 232, 0.075)");
  mwGrad.addColorStop(0.45, "rgba(115, 145, 205, 0.032)");
  mwGrad.addColorStop(1, "rgba(80, 110, 170, 0)");
  ctx.scale(1.6, 0.42);
  ctx.fillStyle = mwGrad;
  ctx.beginPath();
  ctx.arc(0, 0, Math.max(w, h) * 0.62, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  // 星群：远景微星 + 中景冷暖星 + 亮星光晕
  const totalStars = 78;
  for (let i = 0; i < totalStars; i++) {
    const r1 = hash(i * 17 + 3);
    const r2 = hash(i * 17 + 5);
    const r3 = hash(i * 17 + 11);

    // 集中在天幕中上部，越靠近地平线越稀疏柔和
    const x = r1 * w;
    const y = Math.pow(r2, 1.18) * (h * 0.88);
    const horizonFade = 1 - Math.max(0, (y / h - 0.65) / 0.35) * 0.65;

    const speed = 0.55 + r3 * 1.45;
    const phase = i * 3.17;
    const twinkle = 0.38 + 0.62 * (0.5 + 0.5 * Math.sin(t * speed + phase));
    const alpha = twinkle * horizonFade;

    // 区分冷白、暖白、冰蓝三类星色
    const tint =
      i % 5 === 0
        ? "255, 241, 218" // 暖白星
        : i % 3 === 0
          ? "204, 224, 255" // 冰蓝星
          : "236, 242, 252"; // 清冷星

    if (i < 8) {
      // 亮星：带柔和径向光晕与细微星芒
      const radius = 1.35 + r3 * 0.75;
      const halo = ctx.createRadialGradient(x, y, 0, x, y, radius * 5.2);
      halo.addColorStop(0, `rgba(255, 255, 255, ${0.88 * alpha})`);
      halo.addColorStop(0.28, `rgba(${tint}, ${0.34 * alpha})`);
      halo.addColorStop(1, `rgba(${tint}, 0)`);
      ctx.fillStyle = halo;
      ctx.beginPath();
      ctx.arc(x, y, radius * 5.2, 0, Math.PI * 2);
      ctx.fill();
    } else if (i < 30) {
      // 中景星
      const radius = 0.9 + r3 * 0.55;
      ctx.fillStyle = `rgba(${tint}, ${(0.45 + r3 * 0.4) * alpha})`;
      ctx.beginPath();
      ctx.arc(x, y, radius, 0, Math.PI * 2);
      ctx.fill();
    } else {
      // 远景微星
      const radius = 0.55 + r3 * 0.4;
      ctx.fillStyle = `rgba(${tint}, ${(0.22 + r3 * 0.32) * alpha})`;
      ctx.beginPath();
      ctx.arc(x, y, radius, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  // 偶发静谧流星（约每 16 秒划过一次天际上空）
  const cycle = t % 16;
  if (cycle > 4.0 && cycle < 4.9) {
    const p = (cycle - 4.0) / 0.9;
    const easeP = 1 - Math.pow(1 - p, 2);
    const sx = w * 0.74 - easeP * w * 0.26;
    const sy = h * 0.14 + easeP * h * 0.16;
    const tailLen = 58;
    const tx = sx + tailLen * 0.85;
    const ty = sy - tailLen * 0.52;
    const fade = Math.sin(p * Math.PI);

    const meteorGrad = ctx.createLinearGradient(sx, sy, tx, ty);
    meteorGrad.addColorStop(0, `rgba(240, 246, 255, ${0.55 * fade})`);
    meteorGrad.addColorStop(0.4, `rgba(195, 218, 250, ${0.22 * fade})`);
    meteorGrad.addColorStop(1, "rgba(195, 218, 250, 0)");

    ctx.strokeStyle = meteorGrad;
    ctx.lineWidth = 1.2;
    ctx.beginPath();
    ctx.moveTo(sx, sy);
    ctx.lineTo(tx, ty);
    ctx.stroke();
  }
}

/** 晨雾：层叠流动的薄雾云带与晨光微粒 */
function drawFogScene(ctx: CanvasRenderingContext2D, w: number, h: number, t: number) {
  // 多层缓慢横向流动的薄雾带
  for (let band = 0; band < 4; band++) {
    const centerY = h * (0.42 + band * 0.14) + Math.sin(t * (0.22 + band * 0.06) + band * 1.5) * (h * 0.03);
    const bandH = h * (0.18 + band * 0.04);
    const offsetX = Math.sin(t * (0.15 + band * 0.05) + band * 2.1) * (w * 0.12);

    const fogGrad = ctx.createRadialGradient(
      w * 0.5 + offsetX,
      centerY,
      bandH * 0.1,
      w * 0.5 + offsetX,
      centerY,
      Math.max(w * 0.75, bandH * 2.2),
    );
    const alpha = 0.16 + band * 0.035;
    fogGrad.addColorStop(0, `rgba(244, 248, 252, ${alpha})`);
    fogGrad.addColorStop(0.55, `rgba(232, 239, 246, ${alpha * 0.55})`);
    fogGrad.addColorStop(1, "rgba(220, 230, 240, 0)");

    ctx.fillStyle = fogGrad;
    ctx.fillRect(0, 0, w, h);
  }

  // 晨雾中缓缓浮动的微光水汽
  for (let i = 0; i < 20; i++) {
    const r1 = hash(i * 19 + 1);
    const r2 = hash(i * 19 + 3);
    const r3 = hash(i * 19 + 7);
    const x = ((r1 * w + t * (5 + r3 * 6)) % (w + 30)) - 15;
    const y = r2 * h + Math.sin(t * 0.45 + i) * 10;
    const alpha = (0.14 + r3 * 0.16) * (0.5 + 0.5 * Math.sin(t * 0.7 + i));
    const radius = 1.2 + r1 * 1.8;

    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fillStyle = `rgba(255, 255, 255, ${alpha})`;
    ctx.fill();
  }
}
