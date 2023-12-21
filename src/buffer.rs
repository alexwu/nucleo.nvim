#![allow(dead_code)]

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct Window {
    /// The buffer line number of the top of the window
    pos: usize,
    width: usize,
    height: usize,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            width: 10,
            height: 10,
        }
    }
}

impl Window {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            width: x,
            height: y,
            ..Default::default()
        }
    }

    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn set_width(&mut self, width: usize) {
        self.height = width;
    }

    fn set_height(&mut self, height: usize) {
        self.height = height;
    }

    pub fn start(&self) -> usize {
        self.pos
    }

    pub fn end(&self) -> usize {
        self.pos + self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

pub trait Buffer<T: Clone + Debug>: Sized {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn window(&self) -> &Window;
    fn window_mut(&mut self) -> &mut Window;
    fn cursor(&self) -> &Cursor;
    fn cursor_mut(&mut self) -> &mut Cursor;
    fn window_width(&self) -> usize {
        self.window().width()
    }
    fn window_height(&self) -> usize {
        self.window().height()
    }

    fn set_window_width(&mut self, width: usize) {
        self.window_mut().width = width;
    }

    fn set_window_height(&mut self, height: usize) {
        self.window_mut().height = height;
    }

    fn set_window_pos(&mut self, pos: usize) {
        if self.window_height() > self.len() {
            self.window_mut().set_pos(0);
        } else if pos > self.len() - self.window_height() {
            let adjusted_pos = self.len() - self.window_height();
            self.window_mut().set_pos(adjusted_pos);
        } else {
            self.window_mut().set_pos(pos);
        }
    }

    fn clamp_cursor_pos(&mut self, rel: Relative) {
        match rel {
            Relative::Buffer => {
                self.cursor_mut().pos = self.cursor().pos().clamp(0, self.len().saturating_sub(1));
            }
            Relative::Window => {
                let start = self.window().start();
                let end = self.window().end().saturating_sub(1);
                self.cursor_mut().pos = self.cursor().pos().clamp(start, end);
            }
        };
    }

    /// Sets the position of the cursor constrained by the window
    fn set_cursor_pos_in_window(&mut self, pos: usize) {
        let max_pos = self.window().end().min(self.len()).saturating_sub(1);
        log::info!("window max_pos: {}", max_pos);
        let new_pos = pos.saturating_add(self.window().start());
        self.cursor_mut().pos = new_pos.clamp(self.window().start(), max_pos);
    }

    fn get_cursor_pos(&self, rel: Relative) -> usize {
        match rel {
            Relative::Buffer => self.cursor().pos(),
            Relative::Window => self.cursor().pos().saturating_sub(self.window().start()),
        }
    }

    fn set_cursor_pos(&mut self, pos: usize) {
        let new_pos = pos;

        if new_pos >= self.window().end().saturating_sub(1) {
            self.set_window_pos(new_pos.saturating_sub(self.window_height().saturating_sub(1)));
            self.set_cursor_pos_in_window(new_pos);
        } else if new_pos < self.window().start() {
            self.set_window_pos(new_pos);
            self.set_cursor_pos_in_window(new_pos);
        } else {
            self.cursor_mut().pos = new_pos;
        }

        self.clamp_cursor_pos(Relative::Buffer);

        log::info!("buffer cursor pos: {}", self.cursor().pos);
        log::info!(
            "window cursor pos: {}",
            self.get_cursor_pos(Relative::Window)
        );
        log::info!("window pos: {}", self.window().pos);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Relative {
    Buffer,
    Window,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cursor {
    pos: usize,
}

impl Cursor {
    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }
}
