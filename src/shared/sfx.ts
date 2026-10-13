// 界面音效：用 Web Audio 实时合成，不需要音频文件。每个声音都很短、很轻，没有尖锐的高频。

export type Cue = "tap" | "toggle" | "start" | "pause" | "resume" | "end" | "delete";

let ctx: AudioContext | undefined;
let enabled = true;
let volume = 0.4;

export function configureSfx(options: { enabled: boolean; volume: number }) {
  enabled = options.enabled;
  volume = options.volume;
}

function audio(): AudioContext | undefined {
  if (!enabled || volume <= 0 || typeof AudioContext === "undefined") return undefined;
  ctx ??= new AudioContext();
  if (ctx.state === "suspended") void ctx.resume();
  return ctx;
}

interface Tone {
  freq: number;
  at: number;
  dur: number;
  gain: number;
  type?: OscillatorType;
  endFreq?: number;
  attack?: number;
}

function tone(ac: AudioContext, out: AudioNode, t: Tone) {
  const o = ac.createOscillator();
  o.type = t.type ?? "sine";
  o.frequency.setValueAtTime(t.freq, t.at);
  if (t.endFreq) o.frequency.exponentialRampToValueAtTime(t.endFreq, t.at + t.dur);
  const g = ac.createGain();
  g.gain.setValueAtTime(0.0001, t.at);
  g.gain.exponentialRampToValueAtTime(t.gain, t.at + (t.attack ?? 0.004));
  g.gain.exponentialRampToValueAtTime(0.0001, t.at + t.dur);
  o.connect(g).connect(out);
  o.start(t.at);
  o.stop(t.at + t.dur + 0.02);
}

/** 一小段带通滤波的噪声，听起来像木头轻敲的“嗒”。 */
function knock(ac: AudioContext, out: AudioNode, at: number, gain: number, freq: number) {
  const len = Math.floor(ac.sampleRate * 0.03);
  const buffer = ac.createBuffer(1, len, ac.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < len; i++) data[i] = (Math.random() * 2 - 1) * Math.exp(-i / (len / 6));
  const src = ac.createBufferSource();
  src.buffer = buffer;
  const bp = ac.createBiquadFilter();
  bp.type = "bandpass";
  bp.frequency.value = freq;
  bp.Q.value = 1.4;
  const g = ac.createGain();
  g.gain.value = gain;
  src.connect(bp).connect(g).connect(out);
  src.start(at);
}

export function play(cue: Cue) {
  const ac = audio();
  if (!ac) return;
  const out = ac.createGain();
  out.gain.value = volume * 0.6;
  const soften = ac.createBiquadFilter();
  soften.type = "lowpass";
  soften.frequency.value = 3600;
  out.connect(soften).connect(ac.destination);
  const t = ac.currentTime + 0.005;

  switch (cue) {
    case "tap":
      tone(ac, out, { freq: 540, endFreq: 420, at: t, dur: 0.07, gain: 0.45, type: "triangle" });
      knock(ac, out, t, 0.35, 1900);
      break;
    case "toggle":
      knock(ac, out, t, 0.28, 2300);
      knock(ac, out, t + 0.055, 0.2, 2000);
      break;
    case "start":
      tone(ac, out, { freq: 440, at: t, dur: 0.09, gain: 0.4, attack: 0.012 });
      tone(ac, out, { freq: 659, at: t + 0.06, dur: 0.09, gain: 0.4, attack: 0.012 });
      break;
    case "pause":
      tone(ac, out, { freq: 494, at: t, dur: 0.1, gain: 0.38, attack: 0.008 });
      break;
    case "resume":
      tone(ac, out, { freq: 587, at: t, dur: 0.1, gain: 0.38, attack: 0.008 });
      break;
    case "end":
      tone(ac, out, { freq: 659, at: t, dur: 1.4, gain: 0.32, attack: 0.008 });
      tone(ac, out, { freq: 988, at: t, dur: 0.9, gain: 0.1, attack: 0.008 });
      tone(ac, out, { freq: 1318, at: t, dur: 0.5, gain: 0.04, attack: 0.008 });
      break;
    case "delete":
      tone(ac, out, { freq: 200, endFreq: 140, at: t, dur: 0.11, gain: 0.5 });
      break;
  }
}
