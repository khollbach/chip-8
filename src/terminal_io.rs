mod screen;

use self::screen::Screen;
use crate::cpu::io::{Chip8Io, DrawSprite, TIME_BETWEEN_TICKS_NS};
use crate::cpu::screen::Point;
use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    event::{
        self, Event, KeyCode, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::thread;
use std::{
    fmt::{self, Display},
    io, panic,
    time::{Duration, Instant},
};

/// A `crossterm`-based implementation of `Chip8Io`.
#[derive(Debug)]
pub struct TerminalIo {
    screen: Screen,
    kb: KeyboardState,
    previous_tick: Instant,
    dt: u8,
    st: u8,
}

impl TerminalIo {
    // todo: should failure here cause auto-teardown? Think about this
    pub fn setup() -> Result<Self> {
        terminal::enable_raw_mode()?;
        io::stdout()
            .execute(PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
            ))?
            .execute(EnterAlternateScreen)?
            .execute(Clear(ClearType::All))?;

        Ok(Self {
            screen: Screen::default(),
            kb: KeyboardState::default(),
            previous_tick: Instant::now(),
            dt: 0,
            st: 0,
        })
    }

    fn render(&self) -> Result<()> {
        io::stdout()
            .execute(MoveTo(0, 0))?
            .execute(Print(DisplayScreen(&self.screen)))?;
        Ok(())
    }
}

/// Helper for `TerminalIo::render`.
struct DisplayScreen<'a>(&'a Screen);

impl<'a> Display for DisplayScreen<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Debug formatting.
        let s = format!("{:?}", self.0);

        // Translate \n to \r\n to work correctly with raw-mode terminal.
        write!(f, "{}", s.replace('\n', "\r\n"))
    }
}

const TIME_BETWEEN_TICKS: Duration = Duration::from_nanos(TIME_BETWEEN_TICKS_NS);

impl Chip8Io for TerminalIo {
    fn update(&mut self) {
        // Perform new ticks of the delay timer and sound timer.
        //
        // We may end up doing multiple ticks during a single `update`; e.g., if
        // we were blocked waiting for `blocking_get_key`, and a long time
        // passed.

        while self.previous_tick.elapsed() >= TIME_BETWEEN_TICKS {
            self.dt = self.dt.saturating_sub(1);
            self.st = self.st.saturating_sub(1);
            self.previous_tick += TIME_BETWEEN_TICKS;
        }
    }

    fn clear_screen(&mut self) {
        self.screen = Screen::default();
        self.render().unwrap();
    }

    fn get_random_byte(&mut self) -> u8 {
        rand::random()
    }

    fn draw_sprite(&mut self, pos: Point, sprite: &[u8]) -> DrawSprite {
        let collision = self.screen.draw_sprite(pos, sprite);
        self.render().unwrap();

        // Quirk: wait for the next tick of an imaginary "display timer" before returning.
        sleep_until(self.previous_tick + TIME_BETWEEN_TICKS);

        collision
    }

    fn is_key_pressed(&mut self, k: u8) -> bool {
        self.kb.is_key_pressed(k).unwrap()
    }

    fn blocking_get_key(&mut self) -> u8 {
        self.kb.get_key().unwrap()
    }

    fn read_delay_timer(&mut self) -> u8 {
        self.dt
    }

    fn write_delay_timer(&mut self, value: u8) {
        self.dt = value;
    }

    fn write_sound_timer(&mut self, value: u8) {
        self.st = value;
    }
}

impl Drop for TerminalIo {
    fn drop(&mut self) {
        fn try_drop(this: &mut TerminalIo) -> Result<()> {
            // Reset the terminal mode. Otherwise it gets all wonky, and you
            // have to close it and open a new one.
            io::stdout()
                .execute(LeaveAlternateScreen)?
                .execute(PopKeyboardEnhancementFlags)?;
            terminal::disable_raw_mode()?;

            // After leaving the Alternate Screen in the terminal, the text goes away,
            // so we print it again here. This lets us see the last state the screen was
            // in when the emulator exited.
            print!("{:?}", this.screen);
            Ok(())
        }

        // Ignore errors.
        try_drop(self).ok();
    }
}

fn sleep_until(deadline: Instant) {
    thread::sleep(deadline.saturating_duration_since(Instant::now()));
}

// TODO at some point: refactor KeyboardState to fit more harmoniously
// into TerminalIo (todo: how exactly?)

#[derive(Debug, Default)]
struct KeyboardState {
    pressed: [bool; 16],
}

impl KeyboardState {
    fn is_key_pressed(&mut self, x: u8) -> Result<bool> {
        assert!(x <= 0x0f);
        self.consume_pending_input_events()?;
        Ok(self.pressed[x as usize])
    }

    fn consume_pending_input_events(&mut self) -> Result<()> {
        loop {
            if !event::poll(Duration::from_secs(0))? {
                return Ok(());
            }
            self.update_state(&event::read()?);
        }
    }

    fn update_state(&mut self, e: &Event) {
        if let Some((k, pressed)) = filter_event(e) {
            self.pressed[k as usize] = pressed;
        }
    }

    /// Block waiting for one of the 16 keys to be *released*. (This is a
    /// deliberate quirk.)
    fn get_key(&mut self) -> Result<u8> {
        // Catch up on state changes.
        self.consume_pending_input_events()?;

        // Blocking updates, until there's a key release.
        loop {
            let e = event::read()?;
            self.update_state(&e);

            if let Some((k, false)) = filter_event(&e) {
                return Ok(k);
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
