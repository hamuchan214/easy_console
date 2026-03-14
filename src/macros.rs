use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroStep {
    pub send: String,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub steps: Vec<MacroStep>,
}

pub fn expand_escapes(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'r' => {
                    result.push(b'\r');
                    i += 2;
                }
                b'n' => {
                    result.push(b'\n');
                    i += 2;
                }
                b't' => {
                    result.push(b'\t');
                    i += 2;
                }
                b'x' if i + 3 < bytes.len() => {
                    let hex = &s[i + 2..i + 4];
                    if let Ok(byte) = u8::from_str_radix(hex, 16) {
                        result.push(byte);
                        i += 4;
                    } else {
                        result.push(bytes[i]);
                        i += 1;
                    }
                }
                b'\\' => {
                    result.push(b'\\');
                    i += 2;
                }
                _ => {
                    result.push(bytes[i]);
                    i += 1;
                }
            }
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }
    result
}
