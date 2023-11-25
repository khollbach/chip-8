mod keyboard;
mod screen;

use self::keyboard::Keyboard;
use self::screen::Screen;
use crate::cpu::io::{Chip8Io, DrawSprite, TIME_BETWEEN_TICKS_NS};
use crate::cpu::screen::Point;
use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::thread;
use std::{
    fmt::{self, Display},
    io,
    time::{Duration, Instant},
};

/// A `crossterm`-based implementation of `Chip8Io`.
#[derive(Debug)]
pub struct TerminalIo {
    screen: Screen,
    keyboard: Keyboard,
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
            screen: Screen::new(),
            keyboard: Keyboard::default(),
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
        self.keyboard.update().unwrap();

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
        self.screen.clear();
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
        self.keyboard.is_key_pressed(k)
    }

    fn blocking_get_key(&mut self) -> u8 {
        self.keyboard.wait_for_key_release().unwrap()
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
