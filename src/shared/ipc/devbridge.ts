import type { Transport } from "./index";

// 浏览器里只有一个标签页：后端要求打开哪个界面，就跳到对应的 ?view=
const view = new URLSearchParams(window.location.search).get("view") ?? "main";
const label = view === "overlay" ? "overlay-0" : view;

type Handler = (payload: unknown) => void;
const handlers = new Map<string, Set<Handler>>();
let polling = false;

function dispatch(event: string, payload: unknown) {
  if (event === "dev:open") {
    const target = (payload as { view: string }).view;
    if (target !== view) window.location.search = `?view=${target}`;
    return;
  }
  handlers.get(event)?.forEach((h) => h(payload));
}

async function poll() {
  let since = -1;
  for (;;) {
    try {
      const res = await fetch(`/__dev/events?since=${since}`);
      const body = (await res.json()) as { next: number; events: { event: string; payload: unknown }[] };
      if (since >= 0) body.events.forEach((e) => dispatch(e.event, e.payload));
      since = body.next;
    } catch {
      await new Promise((r) => setTimeout(r, 1000));
    }
  }
}

export const devTransport: Transport = {
  async call(cmd, args) {
    const res = await fetch("/__dev/call", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ cmd, args, window: label }),
    });
    const body = (await res.json()) as { ok: boolean; value?: unknown; error?: string };
    if (!body.ok) throw new Error(body.error);
    return body.value;
  },
  listen(event, handler) {
    if (!polling) {
      polling = true;
      void poll();
    }
    let set = handlers.get(event);
    if (!set) handlers.set(event, (set = new Set()));
    set.add(handler);
    return () => set.delete(handler);
  },
};
