import { api } from "../../shared/api";
import { inTauri } from "../../shared/ipc";

/** 选一个形象文件夹或 zip，复制进数据目录。 */
export async function importCharacter() {
  if (inTauri) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const folder = await open({ directory: true, title: "选择形象文件夹" });
    if (typeof folder === "string") {
      await api.character.import(folder);
      return;
    }
    const zip = await open({ title: "或选择 zip", filters: [{ name: "形象包", extensions: ["zip"] }] });
    if (typeof zip === "string") await api.character.import(zip);
    return;
  }
  const file = await new Promise<File | null>((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".zip";
    input.onchange = () => resolve(input.files?.[0] ?? null);
    input.click();
  });
  if (!file) return;
  const res = await fetch("/__dev/import", {
    method: "POST",
    headers: { "x-kind": "character", "x-name": encodeURIComponent(file.name) },
    body: file,
  });
  const body = (await res.json()) as { ok: boolean; error?: string };
  if (!body.ok) throw new Error(body.error ?? "没能导入");
}
