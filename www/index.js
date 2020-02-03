import { Universe, Cell } from "wasm-game-of-life";
import { memory, __wbindgen_malloc, __wbindgen_free } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";


// Construct the universe, and get its width and height.
const universe = Universe.new();
const width = universe.width();
const height = universe.height();

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext("2d");


const ticksSlider = document.getElementById("ticks");
const ticksReadout = document.getElementById("ticks-readout");
ticksSlider.addEventListener("input", event => {
  ticksReadout.textContent = event.target.value;
});
ticksReadout.textContent = ticksSlider.value;


const fpsEl = document.getElementById("fps");

let frames = 0;
let time = Date.now();

let animationId = null;

const renderLoop = () => {
  if (Date.now() - time > 1000) {
    fpsEl.innerText = frames;
    frames = 0;
    time = Date.now();
  } else {
    frames += 1;
  }
  renderOne();
  animationId = requestAnimationFrame(renderLoop);
};

const renderOne = () => {
  universe.tick_many(parseInt(ticksSlider.value, 10));
  draw();
};

const draw = () => {
  drawGrid();
  drawCells();
}

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // Vertical lines.
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }

  // Horizontal lines.
  for (let j = 0; j <= height; j++) {
    ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();
};

const getIndex = (row, column) => {
  return row * width + column;
};

const bitIsSet = (n, arr) => {
  const byte = Math.floor(n / 8);
  const mask = 1 << (n % 8);
  return (arr[byte] & mask) === mask;
};

const drawCells = () => {
  const cellsPtr = universe.cells();
  const cells = new Uint8Array(memory.buffer, cellsPtr, Math.ceil(width * height / 8));

  ctx.beginPath();

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);

      ctx.fillStyle = bitIsSet(idx, cells)
        ? ALIVE_COLOR
        : DEAD_COLOR;

      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }

  ctx.stroke();
};

const isPaused = () => animationId === null;

const playPauseButton = document.getElementById("play-pause");
playPauseButton.addEventListener("click", event => {
  if (isPaused()) {
    if (event.shiftKey) {
      renderOne();
    } else {
      play();
    }
  } else {
    pause();
  }
});

const play = () => {
  playPauseButton.textContent = "⏸";
  renderLoop();
}

const pause = () => {
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
}

canvas.addEventListener("click", event => {
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop = (event.clientY - boundingRect.top) * scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
  const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

  if ((event.ctrlKey || event.metaKey) && event.shiftKey) {
    universe.add_pulsar(row, col);
  } else if (event.ctrlKey || event.metaKey) {
    universe.add_glider(row, col);
  } else {
    universe.toggle_cell(row, col);
  }
  draw();
});

document.getElementById("randomise").addEventListener("click", () => randomise());
document.addEventListener("keydown", event => {
  if (event.key === " ") {
    event.preventDefault();
    randomise();
  }
});

const randomise = () => {
  universe.randomise();
  draw();
};

document.getElementById("clear").addEventListener("click", () => {
  universe.clear();
  draw();
});

document.getElementById("save-current").addEventListener("click", () => save(universe.cells()));

document.getElementById("save-initial").addEventListener(
  "click",
  () => save(universe.initial_cells())
);

const save = cellsPtr => {
  const cells = new Uint8Array(memory.buffer, cellsPtr, Math.ceil(width * height / 8));
  const buffer = cells.buffer.slice(cells.byteOffset, cells.byteLength + cells.byteOffset);
  const blob = new Blob([buffer], {type: "application/octet-stream"});

  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = "life.dat";
  document.body.appendChild(link);
  link.style = "display:none";
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url));
};

const loadInput = document.getElementById("load")
loadInput.addEventListener("change", () => {
  const file = loadInput.files[0];
  const reader = new FileReader();
  reader.addEventListener("loadend", () => {
    const data = new Uint8Array(reader.result);
    const ptr = __wbindgen_malloc(data.length);
    const buffer = new Uint8Array(memory.buffer);
    buffer.set(data, ptr);
    universe.set_state(ptr, data.length);
    __wbindgen_free(ptr, data.length);
    draw();
  });
  reader.readAsArrayBuffer(file);
});

draw();
pause();
