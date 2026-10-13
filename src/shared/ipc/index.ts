// 前端和后端之间的唯一通道。正式运行在 Tauri 里；开发时如果直接用浏览器打开，
// 就改走开发桥（src-tauri/src/bin/devserver.rs），连的是同一套后端逻辑。

export interface Transport {
  call(cmd: string, args: object): Promise<unknown>;
  listen(event: string, handler: (payload: unknown) => void): () => void;
}

export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

let transport: Promise<Transport> | undefined;

function getTransport(): Promise<Transport> {
  transport ??=
    inTauri || !import.meta.env.DEV
      ? import("./tauri").then((m) => m.tauriTransport)
      : import("./devbridge").then((m) => m.devTransport);
  return transport;
}

export async function call<T>(cmd: string, args: object = {}): Promise<T> {
  const t = await getTransport();
  return (await t.call(cmd, args)) as T;
}

export function on<T>(event: string, handler: (payload: T) => void): () => void {
  let off: (() => void) | undefined;
  let cancelled = false;
  void getTransport().then((t) => {
    if (!cancelled) off = t.listen(event, handler as (payload: unknown) => void);
  });
  return () => {
    cancelled = true;
    off?.();
  };
}
