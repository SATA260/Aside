"""把霞鹜文楷屏幕版裁成常用字子集（GB2312 全部 6763 个汉字 + 标点 + ASCII），输出 woff2。

用法：
    python3 -m venv .venv && .venv/bin/pip install fonttools brotli
    .venv/bin/python scripts/subset-wenkai.py LXGWWenKaiGBScreen.ttf src/assets/fonts/lxgw-wenkai-screen.woff2

字体下载：https://github.com/lxgw/LxgwWenKai-Screen/releases（SIL OFL 1.1）
"""

import sys

from fontTools import subset


def gb2312_chars() -> str:
    chars = []
    for hi in range(0xA1, 0xF8):
        for lo in range(0xA1, 0xFF):
            try:
                chars.append(bytes([hi, lo]).decode("gb2312"))
            except UnicodeDecodeError:
                pass
    return "".join(chars)


def main(src: str, dst: str) -> None:
    text = gb2312_chars()
    ranges = [
        range(0x20, 0x7F),  # ASCII
        range(0x2000, 0x2070),  # 通用标点
        range(0x3000, 0x3040),  # 中文标点
        range(0xFF00, 0xFFF0),  # 全角字符
    ]
    unicodes = {ord(c) for c in text} | {u for r in ranges for u in r}

    options = subset.Options()
    options.flavor = "woff2"
    options.layout_features = ["*"]
    options.name_IDs = ["*"]
    options.name_languages = ["*"]
    font = subset.load_font(src, options)
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=unicodes)
    subsetter.subset(font)
    subset.save_font(font, dst, options)
    print(f"{len(unicodes)} 个字符 -> {dst}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
