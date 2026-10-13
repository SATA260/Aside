// 列出屏幕上 Aside 的窗口：编号、位置大小、层级。真机检查窗口时用，不需要录屏权限。
// 用法：swift scripts/windows.swift

import CoreGraphics

let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] ?? []
for w in list where (w[kCGWindowOwnerName as String] as? String ?? "").lowercased() == "aside" {
  let b = w[kCGWindowBounds as String] as? [String: Double] ?? [:]
  let id = w[kCGWindowNumber as String] ?? 0
  let layer = w[kCGWindowLayer as String] ?? 0
  print("window \(id) x=\(b["X"] ?? 0) y=\(b["Y"] ?? 0) w=\(b["Width"] ?? 0) h=\(b["Height"] ?? 0) layer=\(layer)")
}
