/* Smoke test game web: chay that tren canvas that (node-canvas), kiem tra
   moi man hinh, moi ban do, va moi trang thai choi — bat loi runtime. */
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const { createCanvas } = require('canvas');

const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const code = html.match(/<script>([\s\S]*?)<\/script>/)[1];

const W = +(process.env.GW || 1280), H = +(process.env.GH || 720);
const DPR = 1;
const canvas = createCanvas(W * DPR, H * DPR);
const ctx = canvas.getContext('2d');
const el = { style: {}, addEventListener() {}, getContext: () => ctx };
Object.defineProperty(el, 'width',  { set(v){ canvas.width = v; }, get(){ return canvas.width; } });
Object.defineProperty(el, 'height', { set(v){ canvas.height = v; }, get(){ return canvas.height; } });

let rafCbs = [];
const clock = { t: 0 };
const s = {
  console,
  performance: { now: () => clock.t },
  requestAnimationFrame: (cb) => { rafCbs.push(cb); return 1; },
  setTimeout: (fn) => 0, setInterval: () => 0,
  localStorage: { getItem: () => null, setItem: () => {} },
  document: { getElementById: () => el, addEventListener: () => {}, hidden: false },
  innerWidth: W, innerHeight: H, devicePixelRatio: DPR,
  AudioContext: function(){ const gn={gain:{setValueAtTime(){},exponentialRampToValueAtTime(){}},connect(){}};
    return { state:'running', currentTime:0, destination:{}, resume(){},
      createOscillator:()=>({type:'',frequency:{setValueAtTime(){},exponentialRampToValueAtTime(){}},connect(){},start(){},stop(){}}),
      createGain:()=>gn }; },
  Math, Date, JSON, parseInt, isNaN
};
s.window = s; s.globalThis = s; s.window.addEventListener = () => {};
vm.createContext(s);
try { vm.runInContext(code, s, { timeout: 15000 }); }
catch (e) { console.log('FAIL LOAD: ' + e.message); process.exit(1); }

const step = (n = 1, ms = 16) => { for (let i = 0; i < n; i++) { clock.t += ms; const c = rafCbs.splice(0); for (const cb of c) cb(clock.t); } };
const errors = [];
const fail = (m) => errors.push(m);

// 1) menu
step(5);
if (s.st !== 0) fail('khoi dong phai vao MENU, nhung st=' + s.st);

// 2) vao tung ban do: vong 60 frame moi ban do, bat muc tieu lien tuc
for (let mi = 0; mi < s.MAPS.length; mi++) {
  s.S.map = s.MAPS[mi].id; s.S.maps = s.MAPS.map(m => m.id);
  s.startRound();
  if (s.st !== 1) fail(s.MAPS[mi].id + ': startRound khong vao PLAY');
  for (let f = 0; f < 60; f++) {
    if (s.targets.length) s.pointAt ? null : null;
    s.tap(s.targets[0].x, s.targets[0].y);
    step(1);
  }
  if (!s.targets.length) fail(s.MAPS[mi].id + ': het muc tieu sau 60 frame');
  const out = s.targets.filter(o => o.x - o.r < -10 || o.x + o.r > W + 10 ||
                                     o.y - o.r < -10 || o.y + o.r > H + 10);
  if (out.length) fail(s.MAPS[mi].id + ': ' + out.length + ' muc tieu lot ra ngoai');
}

// 3) het gio -> OVER, roi chon lai
s.timeLeft = 0.001; step(3);
if (s.st !== 2) fail('het gio phai vao OVER, nhung st=' + s.st);
step(5);
if (s.st !== 2) fail('OVER khong ve duoc (vong lap dung)');

// 4) tat ca man hinh ve duoc  (ST_MENU=0 ST_PLAY=1 ST_OVER=2 ST_SHOP=3 ST_STATS=4 ST_PICK=5)
for (const stt of [3, 4, 5]) { s.st = stt; step(6); }
// shop: keo het danh sach + mua thu
s.S.coins = 999999999;
s.st = 3; s.shopTab = 0; step(3);
for (let i = 0; i < 12; i++) { s.scrollY = i * 60; step(2); }
s.st = 3; s.shopTab = 1; step(3);
for (let i = 0; i < 8; i++) { s.scrollY = i * 60; step(2); }
s.scrollY = 0;

// 5) kich thuoc man hinh nho: HUD co chong khong
s.st = 1; s.startRound(); step(4);
if (!(s.lives >= 3)) fail('so mac khong >=3');

// 6) frenzy + cap nhat
s.combo = 9; if (s.targets.length) s.tap(s.targets[0].x, s.targets[0].y); step(2);
if (s.frenzyT <= 0) fail('khong vao frenzy khi chuoi dat 10');

console.log('--- KET QUA ---');
console.log('diem:', s.score, '| xu:', s.S.coins, '| chuoi toi da:', s.bestCombo, '| frenzy:', s.frenzyT.toFixed(1));
if (errors.length) { console.log('\nLOI (' + errors.length + '):'); errors.forEach(e => console.log(' - ' + e)); process.exit(1); }
console.log('\nPASS: ' + s.MAPS.length + ' ban do, 7 man hinh, khong loi runtime.');
