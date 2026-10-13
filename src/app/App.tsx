import { MainView } from "../views/main/MainView";
import { OverlayView } from "../views/overlay/OverlayView";
import { MiniView } from "../views/mini/MiniView";
import { LecturePanel } from "../views/lecture/LecturePanel";

const VIEWS = {
  main: MainView,
  overlay: OverlayView,
  mini: MiniView,
  lecture: LecturePanel,
} as const;

type ViewName = keyof typeof VIEWS;

// 每个窗口都加载同一个 index.html，后端创建窗口时用 ?view= 指定显示哪个界面
function currentView(): ViewName {
  const view = new URLSearchParams(window.location.search).get("view");
  return view && view in VIEWS ? (view as ViewName) : "main";
}

export function App() {
  const View = VIEWS[currentView()];
  return <View />;
}
