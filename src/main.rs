use anyhow::Result;
use chip_8::{Chip8, Chip8Io, Screen};
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
use std::{
    cell::RefCell,
    fmt::{self, Display},
    io::{self, Read},
    panic,
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    // Catch panics and errors, so we can reset the terminal mode.
    // Otherwise it gets all wonky, and you have to close it and open a new one.
    let err = panic::catch_unwind(run);
    terminal::disable_raw_mode()?;
    err.unwrap()?;

    Ok(())
}

fn run() -> Result<()> {
    let mut rom = vec![];
    io::stdin().read_to_end(&mut rom)?;

    terminal::enable_raw_mode()?;
    io::stdout()
        .execute(PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
        ))?
        .execute(EnterAlternateScreen)?
        .execute(Clear(ClearType::All))?;

    let mut saved_screen = None;
    let render = |screen: &Screen| {
        render(screen).unwrap();
        saved_screen = Some(screen.clone());
    };

    let kb: RefCell<KeyboardState> = Default::default();
    let is_key_pressed = |k| kb.borrow_mut().is_key_pressed(k).unwrap();
    let get_key = || kb.borrow_mut().get_key().unwrap();

    let mut timer = Timer::new();
    let poll_timer = || timer.poll();

    let io = Chip8Io::new(render, is_key_pressed, get_key, poll_timer);
    Chip8::new(&rom, io).run();

    io::stdout()
        .execute(LeaveAlternateScreen)?
        .execute(PopKeyboardEnhancementFlags)?;
    terminal::disable_raw_mode()?;

    // After leaving the Alternate Screen in the terminal, the text goes away,
    // so we print it again here. This lets us see the last state the screen was
    // in when the emulator exited.
    if let Some(screen) = saved_screen {
        print!("{screen:?}");
    }

    Ok(())
}

struct Timer {
    previous_tick: Instant,
}

impl Timer {
    const FREQ_HZ: f64 = 60.;

    fn new() -> Self {
        Self {
            previous_tick: Instant::now(),
        }
    }

    fn poll(&mut self) -> bool {
        let time_between_ticks = Duration::from_secs_f64(1. / Self::FREQ_HZ);

        if self.previous_tick.elapsed() >= time_between_ticks {
            self.previous_tick = Instant::now();
            true
        } else {
            false
        }
    }
}

fn render(screen: &Screen) -> Result<()> {
    io::stdout()
        .execute(MoveTo(0, 0))?
        .execute(Print(DisplayScreen(screen)))?;
    Ok(())
}

/// Helper for `render`.
struct DisplayScreen<'a>(&'a Screen);

impl<'a> Display for DisplayScreen<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Debug formatting.
        let s = format!("{:?}", self.0);

        // Translate \n to \r\n to work correctly with raw-mode terminal.
        write!(f, "{}", s.replace('\n', "\r\n"))
    }
}

#[derive(Debug, Default)]
struct KeyboardState {
    pressed: [bool; 16],
    most_recently_pressed: u8,
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
            self.update_state(event::read()?);
        }
    }

    fn update_state(&mut self, terminal_event: Event) {
        let Event::Key(e) = terminal_event else {
            return;
        };
        let KeyCode::Char(c) = e.code else { return };
        let pressed = match e.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => true,
            KeyEventKind::Release => false,
        };

        // Bail on ctrl+c.
        //
        // Note that this only gets hit if the program asks for input. One
        // possible fix is to have a separate thread that handles io.
        if matches!(c, 'c' | 'C') && e.modifiers.contains(KeyModifiers::CONTROL) && pressed {
            panic!("control-c pressed");
        }

        if let Some(k) = keycode_to_chip8(c) {
            self.pressed[k as usize] = pressed;
            if pressed {
                self.most_recently_pressed = k;
            }
        }
    }

    /// Block waiting for one of the 16 keys to become pressed, then return the
    /// keycode (0x0..=0xf) of that key.
    fn get_key(&mut self) -> Result<u8> {
        self.consume_pending_input_events()?;

        loop {
            if self.pressed[self.most_recently_pressed as usize] {
                return Ok(self.most_recently_pressed);
            }
            debug_assert!(self.pressed.iter().all(|p| !p));

            self.update_state(event::read()?);
        }
    }
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
