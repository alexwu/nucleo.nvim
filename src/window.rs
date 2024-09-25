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

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn set_width(&mut self, width: usize) {
        self.height = width;
    }

    pub fn set_height(&mut self, height: usize) {
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

    pub fn pos(&self) -> usize {
        self.pos
    }
}
