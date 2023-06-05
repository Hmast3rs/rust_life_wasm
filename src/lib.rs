mod utils;

use fixedbitset::FixedBitSet;
use wasm_bindgen::prelude::*;

use utils::Timer;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    toggle: bool,
    buffer_a: FixedBitSet,
    buffer_b: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let size = (width * height) as usize;
        let mut initial = FixedBitSet::with_capacity(size);
        for i in 0..size {
            initial.set(i, i % 2 == 0 || i % 7 == 0)
        }
        Universe {
            width,
            height,
            toggle: true,
            buffer_a: initial,
            buffer_b: FixedBitSet::with_capacity(size),
        }
    }

    pub fn cells(&self) -> *const u32 {
        let active = if self.toggle {
            &self.buffer_a
        } else {
            &self.buffer_b
        };
        active.as_slice().as_ptr()
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");

        // Use a raw pointer to the inactive buffer to avoid
        // borrowing, so we can borrow self again for other methods
        let (active, inactive) = if self.toggle {
            (&self.buffer_a, &mut self.buffer_b as *mut FixedBitSet)
        } else {
            (&self.buffer_b, &mut self.buffer_a as *mut FixedBitSet)
        };

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = active[idx];
                let count = self.count(active, row, col);

                // Rule 1: Any live cell with fewer than two live neighbours
                // dies, as if caused by underpopulation.
                // Rule 2: Any live cell with two or three live neighbours
                // lives on to the next generation.
                // Rule 3: Any live cell with more than three live
                // neighbours dies, as if by overpopulation.
                // Rule 4: Any dead cell with exactly three live neighbours
                // becomes a live cell, as if by reproduction.
                // All other cells remain in the same state.

                let next = count == 3 || (cell && count == 2);

                unsafe { (*inactive).set(idx, next) }
            }
        }

        self.toggle = !self.toggle;
    }

    fn count(&self, active: &FixedBitSet, row: u32, col: u32) -> u8 {
        let north = if row == 0 { self.height - 1 } else { row - 1 };
        let south = if row == self.height - 1 { 0 } else { row + 1 };
        let west = if col == 0 { self.width - 1 } else { col - 1 };
        let east = if col == self.width - 1 { 0 } else { col + 1 };

        let nw = self.get_index(north, west);
        let nn = self.get_index(north, col);
        let ne = self.get_index(north, east);
        let ww = self.get_index(row, west);
        let ee = self.get_index(row, east);
        let sw = self.get_index(south, west);
        let ss = self.get_index(south, col);
        let se = self.get_index(south, east);

        [
            active[nw] as u8,
            active[nn] as u8,
            active[ne] as u8,
            active[ww] as u8,
            active[ee] as u8,
            active[sw] as u8,
            active[ss] as u8,
            active[se] as u8,
        ]
        .iter()
        .sum()
    }
}
