use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::{panic, time::Duration};

#[derive(Debug, Default)]
pub struct Keyboard {
    pressed: [bool; 16],
}

impl Keyboard {
    pub fn update(&mut self) -> Result<()> {
        // Consume pending input events; update state.
        while event::poll(Duration::from_secs(0))? {
            if let Some((k, pressed)) = filter_event(&event::read()?) {
                self.pressed[k as usize] = pressed;
            }
        }
        Ok(())
    }

    pub fn is_key_pressed(&self, x: u8) -> bool {
        assert!(x <= 0x0f);
        self.pressed[x as usize]
    }

    /// Block waiting for any of the 16 keys to go from pressed to released.
    pub fn wait_for_key_release(&mut self) -> Result<u8> {
        loop {
            if let Some((k, pressed)) = filter_event(&event::read()?) {
                self.pressed[k as usize] = pressed;

                if !pressed {
                    return Ok(k);
                }
            }
        }
    }
}

/// If this is a relevant key-press/release event, return:
/// * `(chip8_keycode, pressed)`
fn filter_event(terminal_event: &Event) -> Option<(u8, bool)> {
    let Event::Key(e) = terminal_event else {
        return None;
    };
    let KeyCode::Char(c) = e.code else {
        return None;
    };
    let pressed = match e.kind {
        KeyEventKind::Press | KeyEventKind::Repeat => true,
        KeyEventKind::Release => false,
    };

    // Hack: bail on ctrl+c.
    //
    // Note that this only gets hit if the program asks for input. One
    // possible fix is to have a separate thread that handles io.
    //
    // TODO: look into this further.
    if matches!(c, 'c' | 'C') && e.modifiers.contains(KeyModifiers::CONTROL) && pressed {
        panic!("control-c pressed");
    }

    let Some(k) = keycode_to_chip8(c) else {
        return None;
    };

    Some((k, pressed))
}

/// Translate a key from the physical keyboard into one of the 16 virtual keys
/// on the CHIP-8.
///
/// I've chosen to map the 4x4 square from `7` through `/` on the physical
/// keyboard. All other keycodes return `None`.
fn keycode_to_chip8(c: char) -> Option<u8> {
    // let key = match c {
    //     '7' | '&' => 0x1,
    //     '8' | '*' => 0x2,
    //     '9' | '(' => 0x3,
    //     'u' | 'U' => 0x4,
    //     'i' | 'I' => 0x5,
    //     'o' | 'O' => 0x6,
    //     'j' | 'J' => 0x7,
    //     'k' | 'K' => 0x8,
    //     'l' | 'L' => 0x9,

    //     'm' | 'M' => 0xa,
    //     ',' | '<' => 0x0,
    //     '.' | '>' => 0xb,

    //     '0' | ')' => 0xc,
    //     'p' | 'P' => 0xd,
    //     ';' | ':' => 0xe,
    //     '/' | '?' => 0xf,

    //     _ => return None,
    // };
    // Some(key)

    // TODO: hacky workaround for my weird keyboard setup.
    // Change this back at some point...
    workman_keycode_to_chip8(c)
}

fn workman_keycode_to_chip8(c: char) -> Option<u8> {
    let key = match c {
        '7' | '&' => 0x1,
        '8' | '*' => 0x2,
        '9' | '(' => 0x3,
        'f' | 'F' => 0x4,
        'u' | 'U' => 0x5,
        'p' | 'P' => 0x6,
        'n' | 'N' => 0x7,
        'e' | 'E' => 0x8,
        'o' | 'O' => 0x9,

        'l' | 'L' => 0xa,
        ',' | '<' => 0x0,
        '.' | '>' => 0xb,

        '0' | ')' => 0xc,
        ';' | ':' => 0xd,
        'i' | 'I' => 0xe,
        '/' | '?' => 0xf,

        _ => return None,
    };
    Some(key)
}
