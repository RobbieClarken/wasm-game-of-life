mod utils;

use fixedbitset::FixedBitSet;
use js_sys;
use wasm_bindgen::prelude::*;
use web_sys;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
    initial_cells: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        utils::set_panic_hook();
        log!("Universe::new()");
        let width = 100;
        let height = 100;
        let cells = Self::random_symmetric(height, width);
        Self {
            width,
            height,
            initial_cells: cells.clone(),
            cells,
        }
    }

    pub fn randomise(&mut self) {
        self.cells = Self::random_symmetric(self.height, self.width);
        self.initial_cells = self.cells.clone();
    }

    pub fn clear(&mut self) {
        let size = (self.width * self.height) as usize;
        self.cells = FixedBitSet::with_capacity(size);
        self.initial_cells = self.cells.clone();
    }

    #[allow(dead_code)]
    fn random_cells(height: u32, width: u32) -> FixedBitSet {
        let spawn_size = 10;
        let spawn_min_x = width / 2 - spawn_size / 2;
        let spawn_max_x = spawn_min_x + spawn_size;
        let spawn_min_y = height / 2 - spawn_size / 2;
        let spawn_max_y = spawn_min_y + spawn_size;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            let column = i as u32 % width;
            let row = i as u32 / width;
            if column < spawn_min_x
                || column > spawn_max_x
                || row < spawn_min_y
                || row > spawn_max_y
            {
                continue;
            }
            cells.set(i, js_sys::Math::random() < 0.5)
        }
        cells
    }

    #[allow(dead_code)]
    fn random_symmetric(height: u32, width: u32) -> FixedBitSet {
        let start = 40;
        let mid_x = width / 2;
        let mid_y = height / 2;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for x in start..mid_x {
            for y in x..mid_y {
                let cell = js_sys::Math::random() < 0.5;

                let i = (y * width + x) as usize;
                cells.set(i, cell);

                let i = (x * width + y) as usize;
                cells.set(i, cell);

                let i = ((height - 1 - y) * width + x) as usize;
                cells.set(i, cell);

                let i = ((height - 1 - x) * width + y) as usize;
                cells.set(i, cell);

                let i = (y * width + (width - 1 - x)) as usize;
                cells.set(i, cell);

                let i = (x * width + (width - 1 - y)) as usize;
                cells.set(i, cell);

                let i = ((height - 1 - y) * width + (width - 1 - x)) as usize;
                cells.set(i, cell);

                let i = ((height - 1 - x) * width + (width - 1 - y)) as usize;
                cells.set(i, cell);
            }
        }
        cells
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = FixedBitSet::with_capacity((width * self.height) as usize);
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = FixedBitSet::with_capacity((self.width * height) as usize);
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    pub fn initial_cells(&self) -> *const u32 {
        self.initial_cells.as_slice().as_ptr()
    }

    pub fn set_state(&mut self, ptr: *const u8, buf_length: usize) {
        let slice = unsafe { std::slice::from_raw_parts(ptr, buf_length) };
        let size = (self.width * self.height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            let byte = i / 8;
            let mask = 1 << (i % 8);
            cells.set(i, (slice[byte] & mask) == mask);
        }
        self.cells = cells;
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        self.tick_many(1);
    }

    pub fn tick_many(&mut self, ticks: usize) {
        for _ in 0..ticks {
            let mut next = self.cells.clone();
            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.cells[idx];
                    let live_neighbors = self.live_neighbor_count(row, col);

                    next.set(
                        idx,
                        match (cell, live_neighbors) {
                            // Rule 1: Any live cell with fewer than two live neighbours
                            // dies, as if caused by underpopulation.
                            (true, x) if x < 2 => false,
                            // Rule 2: Any live cell with two or three live neighbours
                            // lives on to the next generation.
                            (true, 2) | (true, 3) => true,
                            // Rule 3: Any live cell with more than three live
                            // neighbours dies, as if by overpopulation.
                            (true, x) if x > 3 => false,
                            // Rule 4: Any dead cell with exactly three live neighbours
                            // becomes a live cell, as if by reproduction.
                            (false, 3) => true,
                            // All other cells remain in the same state.
                            (otherwise, _) => otherwise,
                        },
                    );
                }
            }
            self.cells = next;
        }
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, !self.cells[idx]);
    }

    pub fn add_glider(&mut self, row: u32, col: u32) {
        self.set_cells(&[
            (row as i32 - 2, col as i32 - 1),
            (row as i32 - 1, col as i32),
            (row as i32, col as i32 - 2),
            (row as i32, col as i32 - 1),
            (row as i32, col as i32),
        ]);
    }

    pub fn add_pulsar(&mut self, row: u32, col: u32) {
        for (q_row, q_col) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            self.set_cells(&[
                (row as i32 + q_row * 1, col as i32 + q_col * 2),
                (row as i32 + q_row * 1, col as i32 + q_col * 3),
                (row as i32 + q_row * 1, col as i32 + q_col * 4),
                (row as i32 + q_row * 2, col as i32 + q_col * 1),
                (row as i32 + q_row * 3, col as i32 + q_col * 1),
                (row as i32 + q_row * 4, col as i32 + q_col * 1),
                (row as i32 + q_row * 6, col as i32 + q_col * 2),
                (row as i32 + q_row * 6, col as i32 + q_col * 3),
                (row as i32 + q_row * 6, col as i32 + q_col * 4),
                (row as i32 + q_row * 2, col as i32 + q_col * 6),
                (row as i32 + q_row * 3, col as i32 + q_col * 6),
                (row as i32 + q_row * 4, col as i32 + q_col * 6),
            ]);
        }
    }
}

impl Universe {
    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &fixedbitset::FixedBitSet {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(i32, i32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(
                row.rem_euclid(self.height as i32) as u32,
                col.rem_euclid(self.width as i32) as u32,
            );
            self.cells.set(idx, true);
        }
    }
}
