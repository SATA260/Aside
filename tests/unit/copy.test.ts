import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { expect, it } from "vitest";

// 界面文案的底线：不评判、不催促
const FORBIDDEN = ["失败", "未完成", "落后", "只学了", "还差"];

const root = join(__dirname, "../..");

function files(dir: string, exts: string[]): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return files(path, exts);
    return exts.some((e) => name.endsWith(e)) ? [path] : [];
  });
}

it("user-facing text never uses judging words", () => {
  const hits: string[] = [];
  const sources = [...files(join(root, "src"), [".ts", ".tsx", ".css", ".json"]), ...files(join(root, "src-tauri/src"), [".rs"])];
  for (const file of sources) {
    readFileSync(file, "utf8")
      .split("\n")
      .forEach((line, i) => {
        // 只写进日志的错误信息用户看不到
        if (file.endsWith(".rs") && line.includes("eprintln!")) return;
        for (const word of FORBIDDEN) {
          if (line.includes(word)) hits.push(`${relative(root, file)}:${i + 1} “${word}”`);
        }
      });
  }
  expect(hits).toEqual([]);
});
