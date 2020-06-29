// Copyright 2020 The druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Web keycode handling.

use web_sys::KeyboardEvent;

use crate::keyboard::{Code, KbKey, KeyEvent, KeyState, Location, Modifiers};

/// Convert a web-sys KeyboardEvent into a keyboard-types one.
pub(crate) fn convert_keyboard_event(
    event: &KeyboardEvent,
    mods: Modifiers,
    state: KeyState,
) -> KeyEvent {
    KeyEvent {
        state,
        key: event.key().parse().unwrap_or(KbKey::Unidentified),
        code: convert_code(&event.code()),
        location: convert_location(event.location()),
        mods,
        repeat: event.repeat(),
        is_composing: event.is_composing(),
    }
}

fn convert_code(code: &str) -> Code {
    match code {
        "Backquote" => Code::Backquote,
        "Backslash" => Code::Backslash,
        "BracketLeft" => Code::BracketLeft,
        "BracketRight" => Code::BracketRight,
        "Comma" => Code::Comma,
        "Digit0" => Code::Digit0,
        "Digit1" => Code::Digit1,
        "Digit2" => Code::Digit2,
        "Digit3" => Code::Digit3,
        "Digit4" => Code::Digit4,
        "Digit5" => Code::Digit5,
        "Digit6" => Code::Digit6,
        "Digit7" => Code::Digit7,
        "Digit8" => Code::Digit8,
        "Digit9" => Code::Digit9,
        "Equal" => Code::Equal,
        "IntlBackslash" => Code::IntlBackslash,
        "IntlRo" => Code::IntlRo,
        "IntlYen" => Code::IntlYen,
        "KeyA" => Code::KeyA,
        "KeyB" => Code::KeyB,
        "KeyC" => Code::KeyC,
        "KeyD" => Code::KeyD,
        "KeyE" => Code::KeyE,
        "KeyF" => Code::KeyF,
        "KeyG" => Code::KeyG,
        "KeyH" => Code::KeyH,
        "KeyI" => Code::KeyI,
        "KeyJ" => Code::KeyJ,
        "KeyK" => Code::KeyK,
        "KeyL" => Code::KeyL,
        "KeyM" => Code::KeyM,
        "KeyN" => Code::KeyN,
        "KeyO" => Code::KeyO,
        "KeyP" => Code::KeyP,
        "KeyQ" => Code::KeyQ,
        "KeyR" => Code::KeyR,
        "KeyS" => Code::KeyS,
        "KeyT" => Code::KeyT,
        "KeyU" => Code::KeyU,
        "KeyV" => Code::KeyV,
        "KeyW" => Code::KeyW,
        "KeyX" => Code::KeyX,
        "KeyY" => Code::KeyY,
        "KeyZ" => Code::KeyZ,
        "Minus" => Code::Minus,
        "Period" => Code::Period,
        "Quote" => Code::Quote,
        "Semicolon" => Code::Semicolon,
        "Slash" => Code::Slash,
        "AltLeft" => Code::AltLeft,
        "AltRight" => Code::AltRight,
        "Backspace" => Code::Backspace,
        "CapsLock" => Code::CapsLock,
        "ContextMenu" => Code::ContextMenu,
        "ControlLeft" => Code::ControlLeft,
        "ControlRight" => Code::ControlRight,
        "Enter" => Code::Enter,
        "MetaLeft" => Code::MetaLeft,
        "MetaRight" => Code::MetaRight,
        "ShiftLeft" => Code::ShiftLeft,
        "ShiftRight" => Code::ShiftRight,
        "Space" => Code::Space,
        "Tab" => Code::Tab,
        "Convert" => Code::Convert,
        "KanaMode" => Code::KanaMode,
        "Lang1" => Code::Lang1,
        "Lang2" => Code::Lang2,
        "Lang3" => Code::Lang3,
        "Lang4" => Code::Lang4,
        "Lang5" => Code::Lang5,
        "NonConvert" => Code::NonConvert,
        "Delete" => Code::Delete,
        "End" => Code::End,
        "Help" => Code::Help,
        "Home" => Code::Home,
        "Insert" => Code::Insert,
        "PageDown" => Code::PageDown,
        "PageUp" => Code::PageUp,
        "ArrowDown" => Code::ArrowDown,
        "ArrowLeft" => Code::ArrowLeft,
        "ArrowRight" => Code::ArrowRight,
        "ArrowUp" => Code::ArrowUp,
        "NumLock" => Code::NumLock,
        "Numpad0" => Code::Numpad0,
        "Numpad1" => Code::Numpad1,
        "Numpad2" => Code::Numpad2,
        "Numpad3" => Code::Numpad3,
        "Numpad4" => Code::Numpad4,
        "Numpad5" => Code::Numpad5,
        "Numpad6" => Code::Numpad6,
        "Numpad7" => Code::Numpad7,
        "Numpad8" => Code::Numpad8,
        "Numpad9" => Code::Numpad9,
        "NumpadAdd" => Code::NumpadAdd,
        "NumpadBackspace" => Code::NumpadBackspace,
        "NumpadClear" => Code::NumpadClear,
        "NumpadClearEntry" => Code::NumpadClearEntry,
        "NumpadComma" => Code::NumpadComma,
        "NumpadDecimal" => Code::NumpadDecimal,
        "NumpadDivide" => Code::NumpadDivide,
        "NumpadEnter" => Code::NumpadEnter,
        "NumpadEqual" => Code::NumpadEqual,
        "NumpadHash" => Code::NumpadHash,
        "NumpadMemoryAdd" => Code::NumpadMemoryAdd,
        "NumpadMemoryClear" => Code::NumpadMemoryClear,
        "NumpadMemoryRecall" => Code::NumpadMemoryRecall,
        "NumpadMemoryStore" => Code::NumpadMemoryStore,
        "NumpadMemorySubtract" => Code::NumpadMemorySubtract,
        "NumpadMultiply" => Code::NumpadMultiply,
        "NumpadParenLeft" => Code::NumpadParenLeft,
        "NumpadParenRight" => Code::NumpadParenRight,
        "NumpadStar" => Code::NumpadStar,
        "NumpadSubtract" => Code::NumpadSubtract,
        "Escape" => Code::Escape,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        "Fn" => Code::Fn,
        "FnLock" => Code::FnLock,
        "PrintScreen" => Code::PrintScreen,
        "ScrollLock" => Code::ScrollLock,
        "Pause" => Code::Pause,
        "BrowserBack" => Code::BrowserBack,
        "BrowserFavorites" => Code::BrowserFavorites,
        "BrowserForward" => Code::BrowserForward,
        "BrowserHome" => Code::BrowserHome,
        "BrowserRefresh" => Code::BrowserRefresh,
        "BrowserSearch" => Code::BrowserSearch,
        "BrowserStop" => Code::BrowserStop,
        "Eject" => Code::Eject,
        "LaunchApp1" => Code::LaunchApp1,
        "LaunchApp2" => Code::LaunchApp2,
        "LaunchMail" => Code::LaunchMail,
        "MediaPlayPause" => Code::MediaPlayPause,
        "MediaSelect" => Code::MediaSelect,
        "MediaStop" => Code::MediaStop,
        "MediaTrackNext" => Code::MediaTrackNext,
        "MediaTrackPrevious" => Code::MediaTrackPrevious,
        "Power" => Code::Power,
        "Sleep" => Code::Sleep,
        "AudioVolumeDown" => Code::AudioVolumeDown,
        "AudioVolumeMute" => Code::AudioVolumeMute,
        "AudioVolumeUp" => Code::AudioVolumeUp,
        "WakeUp" => Code::WakeUp,
        "Hyper" => Code::Hyper,
        "Super" => Code::Super,
        "Turbo" => Code::Turbo,
        "Abort" => Code::Abort,
        "Resume" => Code::Resume,
        "Suspend" => Code::Suspend,
        "Again" => Code::Again,
        "Copy" => Code::Copy,
        "Cut" => Code::Cut,
        "Find" => Code::Find,
        "Open" => Code::Open,
        "Paste" => Code::Paste,
        "Props" => Code::Props,
        "Select" => Code::Select,
        "Undo" => Code::Undo,
        "Hiragana" => Code::Hiragana,
        "Katakana" => Code::Katakana,
        // Should be exhaustive but in case not, use reasonable default
        _ => Code::Unidentified,
    }
}

fn convert_location(loc: u32) -> Location {
    match loc {
        KeyboardEvent::DOM_KEY_LOCATION_LEFT => Location::Left,
        KeyboardEvent::DOM_KEY_LOCATION_RIGHT => Location::Right,
        KeyboardEvent::DOM_KEY_LOCATION_NUMPAD => Location::Numpad,
        // Should be exhaustive but in case not, use reasonable default
        _ => Location::Standard,
    }
}
