import { type ComponentType, useEffect, useRef, useState } from "react";
import { play } from "../../shared/sfx";
import { useApp } from "../../shared/state";
import "./main.css";
import { SettingsPage } from "./SettingsPage";
import { SubjectsPage } from "./SubjectsPage";

export type PageName = "start" | "records" | "subjects" | "settings";

const PAGES: { name: PageName; label: string; component: ComponentType }[] = [
  { name: "start", label: "开始", component: () => <div className="page" /> },
  { name: "records", label: "记录", component: () => <div className="page" /> },
  { name: "subjects", label: "科目", component: SubjectsPage },
  { name: "settings", label: "设置", component: SettingsPage },
];

export function MainView() {
  const [page, setPage] = useState<PageName>("start");
  const phase = useApp((s) => s.timer?.phase);

  // 一段专注结束、回到电脑时，主窗口总是回到开始页
  const prevPhase = useRef(phase);
  useEffect(() => {
    if (prevPhase.current === "finished" && phase === "idle") setPage("start");
    prevPhase.current = phase;
  }, [phase]);

  const Page = PAGES.find((p) => p.name === page)!.component;
  return (
    <div className="main-window">
      <nav className="main-nav" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          Aside<span> 一旁</span>
        </div>
        {PAGES.map((p) => (
          <button
            type="button"
            key={p.name}
            className={p.name === page ? "nav-item on" : "nav-item"}
            onClick={() => {
              if (p.name !== page) play("tap");
              setPage(p.name);
            }}
          >
            {p.label}
          </button>
        ))}
      </nav>
      <main className="main-content">
        <div className="drag-strip" data-tauri-drag-region />
        <Page />
      </main>
    </div>
  );
}
