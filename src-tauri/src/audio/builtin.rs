//! 内置的四段背景音，编译进程序里，来源和授权见 `src/assets/sounds/CREDITS.md`。

use std::io::Cursor;

use rodio::Source;

pub struct Builtin {
    pub id: &'static str,
    pub name: &'static str,
    bytes: &'static [u8],
}

const BUILTINS: &[Builtin] = &[
    Builtin { id: "rain", name: "雨声", bytes: include_bytes!("../../../src/assets/sounds/rain.wav") },
    Builtin { id: "cafe", name: "咖啡馆", bytes: include_bytes!("../../../src/assets/sounds/cafe.wav") },
    Builtin { id: "fire", name: "篝火", bytes: include_bytes!("../../../src/assets/sounds/fire.wav") },
    Builtin { id: "noise", name: "白噪音", bytes: include_bytes!("../../../src/assets/sounds/noise.wav") },
];

pub fn list() -> &'static [Builtin] {
    BUILTINS
}

pub fn get(id: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.id == id)
}

/// 解码成单声道采样。文件有问题时返回空。
pub fn samples(sound: &Builtin) -> Vec<f32> {
    let Ok(dec) = rodio::Decoder::new(Cursor::new(sound.bytes)) else {
        return Vec::new();
    };
    let channels = dec.channels() as usize;
    let interleaved: Vec<f32> = dec.map(|s| s as f32 / 32768.0).collect();
    if channels <= 1 {
        return interleaved;
    }
    interleaved.chunks(channels).map(|c| c.iter().sum::<f32>() / channels as f32).collect()
}
