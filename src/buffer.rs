#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct Window {
    /// The buffer line number of the top of the window
    pos: usize,
    height: usize,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            height: 10,
        }
    }
}

impl Window {
    pub fn new(height: usize) -> Self {
        Self {
            height,
            ..Default::default()
        }
    }

    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
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
}

pub trait Contents {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait BufferContents<T: Clone>: Contents + Sized {
    fn lines(&self) -> Vec<T>;
    fn window(&self) -> &Window;
    fn window_mut(&mut self) -> &mut Window;
    fn cursor(&self) -> &Cursor;
    fn cursor_mut(&mut self) -> &mut Cursor;
    fn window_height(&self) -> usize {
        self.window().height
    }
    fn set_window_height(&mut self, height: usize) {
        self.window_mut().height = height;
    }

    fn visible_lines(&self) -> Vec<T> {
        let start = self.window().start();
        let end = self.len().min(self.window().end());

        self.lines()[start..end].to_vec()
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
        self.cursor_mut().pos = pos.clamp(self.window().start(), max_pos);
    }

    fn get_cursor_pos(&self, rel: Relative) -> usize {
        match rel {
            Relative::Buffer => self.cursor().pos(),
            Relative::Window => self.cursor().pos().saturating_sub(self.window().start()),
        }
    }

    fn set_cursor_pos(&mut self, pos: usize) {
        let max_pos = self.window_height().min(self.len().saturating_sub(1));
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

        log::info!("window height: {}", self.window_height());
        log::info!("window pos: {}", self.window().pos);
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Buffer<T> {
//     window: Window,
//     cursor: Cursor,
//     lines: Vec<T>,
//     // lines: Arc<T>,
// }
//
// impl<T> Buffer<T> {
//     pub fn new(lines: Vec<T>, window_height: usize) -> Self {
//         Self {
//             window: Window::new(window_height),
//             cursor: Cursor::default(),
//             lines,
//         }
//     }
//
//     fn lines(&self) -> &[T] {
//         &self.lines
//     }
//
//     fn with(&mut self, lines: Vec<T>) {
//         self.lines = lines;
//     }
//
//     fn visible_lines(&self) -> &[T] {
//         let start = self.window.start();
//         let end = self.lines.len().min(self.window.end());
//
//         &self.lines[start..end]
//     }
//
//     fn len(&self) -> usize {
//         self.lines.len()
//     }
//
//     fn is_empty(&self) -> bool {
//         self.lines.is_empty()
//     }
//
//     fn set_window_pos(&mut self, pos: usize) {
//         if self.window.height > self.lines.len() {
//             self.window.pos = 0;
//         } else if pos > self.lines.len() - self.window.height {
//             self.window.pos = self.lines.len() - self.window.height;
//         } else {
//             self.window.pos = pos;
//         }
//     }
//
//     pub fn set_window_height(&mut self, height: usize) {
//         self.window.height = height;
//     }
//
//     /// Sets the position of the cursor constrained by the window
//     fn set_cursor_pos_in_window(&mut self, pos: usize) {
//         let max_pos = self.window.height.min(self.len() - 1);
//         self.cursor.pos = pos.clamp(self.window.start(), max_pos);
//     }
//
//     pub fn get_pos(&self, rel: Relative) -> usize {
//         match rel {
//             Relative::Buffer => self.cursor.pos,
//             Relative::Window => self.cursor.pos.saturating_sub(self.window.start()),
//         }
//     }
//
//     /// Sets the position of the cursor within the buffer, moving the window if necessary
//     pub fn set_cursor_pos(&mut self, pos: usize) {
//         if pos > self.window.end() {
//             self.set_window_pos(pos - self.window.height);
//             self.set_cursor_pos_in_window(pos);
//         } else if pos < self.window.start() {
//             self.set_window_pos(pos);
//             self.set_cursor_pos_in_window(pos);
//         } else {
//             self.cursor.pos = pos;
//         }
//     }
// }
//
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Relative {
    Buffer,
    Window,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cursor {
    // window: Window,
    // buffer: Box<Buffer<T>>,
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

// impl<T: Default> Cursor<T> {
//     pub fn get_pos(&self, rel: Relative) -> usize {
//         match rel {
//             Relative::Buffer => self.pos,
//             Relative::Window => self.pos.saturating_sub(self.window.start()),
//         }
//     }
//     /// Sets the position of the cursor constrained by the window
//     fn set_pos_in_window(&mut self, pos: usize) {
//         let max_pos = self.window.height.min(self.buffer.len() - 1);
//         self.pos = pos.clamp(self.window.start(), max_pos);
//     }
//
//     /// Sets the position of the cursor within the buffer, moving the window if necessary
//     fn set_pos_in_buffer(&mut self, pos: usize) {
//         if pos > self.window.end() {
//             self.buffer.set_window_pos(pos - self.window.height);
//             self.set_pos_in_window(pos);
//         } else if pos < self.window.start() {
//             self.buffer.set_window_pos(pos);
//             self.set_pos_in_window(pos);
//         } else {
//             self.pos = pos;
//         }
//     }
// }
