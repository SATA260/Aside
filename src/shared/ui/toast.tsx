import { create } from "zustand";

interface ToastState {
  items: { id: number; text: string }[];
}

const useToasts = create<ToastState>(() => ({ items: [] }));
let seq = 0;

export function toast(text: string) {
  const id = ++seq;
  useToasts.setState((s) => ({ items: [...s.items, { id, text }] }));
  setTimeout(() => useToasts.setState((s) => ({ items: s.items.filter((t) => t.id !== id) })), 3200);
}

/** 调用后端出错时把提示显示出来，不抛给界面。 */
export async function attempt<T>(p: Promise<T>): Promise<T | undefined> {
  try {
    return await p;
  } catch (err) {
    toast(err instanceof Error ? err.message : String(err));
    return undefined;
  }
}

export function Toaster() {
  const items = useToasts((s) => s.items);
  return (
    <div className="toasts">
      {items.map((t) => (
        <div key={t.id} className="toast">
          {t.text}
        </div>
      ))}
    </div>
  );
}
