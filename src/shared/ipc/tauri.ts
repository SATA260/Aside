import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Transport } from "./index";

export const tauriTransport: Transport = {
  async call(cmd, args) {
    try {
      return await invoke("call", { cmd, args });
    } catch (err) {
      throw new Error(String(err));
    }
  },
  listen(event, handler) {
    const unlisten = listen(event, (e) => handler(e.payload));
    return () => void unlisten.then((fn) => fn());
  },
};
