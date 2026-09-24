/* Kiem tra bo cuc tren nhieu kich thuoc: 6 ban do + 5 man hinh. */
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const { createCanvas } = require('canvas');

const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const code = html.match(/<script>([\s\S]*?)<\/script>/)[1];

const SIZES = [
  { n: 'mobile-portrait',  w: 390,  h: 844, dpr: 3 },
  { n: 'mobile-landscape', w: 844,  h: 390, dpr: 3 },
  { n: 'small',            w: 320,  h: 568, dpr: 2 },
  { n: 'tablet',           w: 820,  h: 1180, dpr: 2 },
  { n: 'desktop',          w: 1280, h: 720,  dpr: 1 },
  { n: 'wide',             w: 1920, h: 1080, dpr: 1 }
];

function boot(W, H, DPR) {
  const canvas = createCanvas(W * DPR, H * DPR);
  const ctx = canvas.getContext('2d');
  const el = { style: {}, addEventListener() {}, getContext: () => ctx };
  Object.defineProperty(el, 'width',  { set(v){ canvas.width = v; }, get(){ return canvas.width; } });
  Object.defineProperty(el, 'height', { set(v){ canvas.height = v; }, get(){ return canvas.height; } });
  let raf = [];
  const clock = { t: 0 };
  const s = {
    console,
    performance: { now: () => clock.t },
    requestAnimationFrame: (cb) => { raf.push(cb); return 1; },
    setTimeout: () => 0, setInterval: () => 0,
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
  vm.runInContext(code, s, { timeout: 15000 });
  s.step = (n=1) => { for (let i=0;i<n;i++){ clock.t += 16; const c=raf.splice(0); for(const cb of c) cb(clock.t); } };
  return s;
}

let bad = 0;
for (const sz of SIZES) {
  const s = boot(sz.w, sz.h, sz.dpr);
  s.step(4);
  const issues = [];

  // moi ban do: muc tieu phai nam trong khung
  for (let mi = 0; mi < s.MAPS.length; mi++) {
    s.S.map = s.MAPS[mi].id;
    s.startRound();
    s.step(90);
    const out = s.targets.filter(o => o.x-o.r < -5 || o.x+o.r > sz.w+5 || o.y-o.r < -5 || o.y+o.r > sz.h+5);
    if (out.length) issues.push(s.MAPS[mi].id + ': ' + out.length + ' muc tieu lot ra ngoai');
  }
  // moi man hinh phai ve duoc va khong nem loi
  for (const stv of [0, 2, 3, 4, 5]) {
    s.st = stv;
    try { s.step(4); } catch (e) { issues.push('st=' + stv + ': ' + e.message); }
  }
  // HUD khong chong chu tren man hinh hep
  s.st = 1; s.startRound(); s.step(4);
  const hitLeft = s.targets.filter(o => o.x-o.r < 130 && o.y-o.r < 95);
  if (hitLeft.length) issues.push(hitLeft.length + ' muc tieu tran vung HUD');

  const tag = String(sz.w)+'x'+String(sz.h);
  console.log(sz.n.padEnd(18), tag.padEnd(10), issues.length ? 'LOI: '+issues.join(' | ') : 'OK');
  if (issues.length) bad++;
}
console.log(bad ? '\nFAIL: ' + bad + '/' + SIZES.length + ' kich thuoc bi loi' : '\nPASS: ' + SIZES.length + ' kich thuoc × ' + 6 + ' ban do × 5 man hinh — sach loai.');
