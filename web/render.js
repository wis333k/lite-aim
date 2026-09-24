/* Render that: chay game tren canvas that, xuat PNG de kiem tra bang mat. */
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const { createCanvas } = require('canvas');

const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const code = html.match(/<script>([\s\S]*?)<\/script>/)[1];

const W = 1280, H = 720;
const canvas = createCanvas(W, H);
const ctx = canvas.getContext('2d');

// thu thap frame de xuat tung anh
const frames = [];
let rafCbs = [];
const realNow = { t: 0 };

const sandbox = {
  console,
  performance: { now: () => realNow.t },
  requestAnimationFrame: (cb) => { rafCbs.push(cb); return rafCbs.length; },
  setTimeout: (fn) => 0,
  localStorage: { getItem: () => null, setItem: () => {} },
  document: {
    getElementById: () => ({ getContext: () => ctx, addEventListener: () => {}, style: {} }),
    addEventListener: () => {},
    hidden: false
  },
  innerWidth: W,
  innerHeight: H,
  devicePixelRatio: 1,
  AudioContext: function () {
    const g0 = { gain: { setValueAtTime(){}, exponentialRampToValueAtTime(){} }, connect(){} };
    return {
      state: 'running', currentTime: 0, destination: {},
      resume() {},
      createOscillator: () => ({ type:'', frequency:{ setValueAtTime(){}, exponentialRampToValueAtTime(){} }, connect(){}, start(){}, stop(){} }),
      createGain: () => g0
    };
  },
  Math, Date, JSON, parseInt, isNaN
};
sandbox.window = sandbox;
sandbox.globalThis = sandbox;
sandbox.window.addEventListener = () => {};

vm.createContext(sandbox);
vm.runInContext(code, sandbox, { timeout: 10000 });

function step(ms) {
  realNow.t += ms;
  const cbs = rafCbs.splice(0);
  for (const cb of cbs) cb(realNow.t);
}
function shot(name) {
  const buf = canvas.toBuffer('image/png');
  fs.writeFileSync(path.join(__dirname, name), buf);
  console.log('da luu', name, (buf.length/1024).toFixed(0)+'KB');
}

// 1) MENU
for (let i=0;i<5;i++) step(16);
shot('shot-menu.png');

// 2) GAMEPLAY ngay khi bat dau
sandbox.startGame();
for (let i=0;i<30;i++) step(16);
shot('shot-play.png');

// 3) GAMEPLAY sau khi cham vai muc tieu (diem > 0, hat bay, chuoi)
for (let n=0;n<6;n++){
  if (sandbox.targets.length) {
    const t = sandbox.targets[0];
    sandbox.pointAt(t.x, t.y);
  }
  step(90);
}
shot('shot-score.png');

// 4) HET MANG
sandbox.timeLeft = 0.001;
step(50);
step(50);
shot('shot-over.png');

console.log('diem cuoi:', sandbox.score, '| best combo:', sandbox.bestCombo, '| trang thai:', sandbox.st);
