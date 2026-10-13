// 全局样式要先于各页面的样式引入，页面样式才能覆盖它
import "../shared/theme/fonts.css";
import "../shared/theme/tokens.css";
import "../shared/theme/base.css";
import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";

// 字体是用到时才加载的，这里提前加载，避免第一次显示时字形跳变
void document.fonts.load('16px "LXGW WenKai Screen"');
void document.fonts.load('200 16px "Manrope"');

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
