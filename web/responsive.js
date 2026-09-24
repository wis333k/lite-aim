/* Kiem tra bo cuc tren man hinh dien thoai (dung + ngang). Canvas that ton trong
   khi gia co DPR, giong het trinh duyet that. */
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const { createCanvas } = require('canvas');

const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const code = html.match(/<script>([\s\S]*?)<\/script>/)[1];

const SIZES = [
  { n: 'mobile-portrait',  w: 390,  h: 844, dpr: 3 },
  { n: 'mobile-landscape', w: 844,  h: 390, dpr: 3 },
  { n: 'tablet',           w: 820,  h: 1180, dpr: 2 },
  { n: 'desktop',          w: 1280, h: 720,  dpr: 1 }
];

function run(W, H, DPR, prefix) {
  const canvas = createCanvas(W * DPR, H * DPR);
  const ctx = canvas.getContext('2d');
  // canvas gia: tu resize theo gia tri game dat vao
  const el = { style: {}, addEventListener() {}, getContext: () => ctx };
  Object.defineProperty(el, 'width',  { set(v){ canvas.width = v; }, get(){ return canvas.width; } });
  Object.defineProperty(el, 'height', { set(v){ canvas.height = v; }, get(){ return canvas.height; } });

  let rafCbs = [];
  const t = { v: 0 };
  const s = {
    console,
    performance: { now: () => t.v },
    requestAnimationFrame: (cb) => { rafCbs.push(cb); return 1; },
    setTimeout: () => 0,
    localStorage: { getItem: () => null, setItem: () => {} },
    document: { getElementById: () => el, addEventListener: () => {}, hidden: false },
    innerWidth: W, innerHeight: H, devicePixelRatio: DPR,
    AudioContext: function(){ return { state:'running', currentTime:0, destination:{},
      resume(){}, createOscillator:()=>({type:'',frequency:{setValueAtTime(){},exponentialRampToValueAtTime(){}},connect(){},start(){},stop(){}}),
      createGain:()=>({gain:{setValueAtTime(){},exponentialRampToValueAtTime(){}},connect(){}}) }; },
    Math, Date, JSON, parseInt, isNaN
  };
  s.window = s; s.globalThis = s; s.window.addEventListener = () => {};
  vm.createContext(s);
  vm.runInContext(code, s, { timeout: 10000 });

  const step = (ms) => { t.v += ms; const c = rafCbs.splice(0); for (const cb of c) cb(t.v); };
  const save = (name) => fs.writeFileSync(path.join(__dirname, prefix + name + '.png'),
                                          canvas.toBuffer('image/png'));

  for (let i = 0; i < 3; i++) step(16);
  save('menu');

  s.startGame();
  for (let i = 0; i < 40; i++) step(16);
  save('play');

  const out = s.targets.map(o => ({ x:o.x, y:o.y, r:o.r, kind:o.kind }));
  // 5 giay: xem con dung khong
  for (let i = 0; i < 300; i++) step(16);
  return out;
}

let bad = 0;
for (const sz of SIZES) {
  const ts = run(sz.w, sz.h, sz.dpr, 'r-' + sz.n + '-');
  const out = ts.filter(o => o.x - o.r < -10 || o.x + o.r > sz.w + 10 ||
                             o.y - o.r < -10 || o.y + o.r > sz.h + 10);
  console.log(sz.n.padEnd(18), (sz.w+'x'+sz.h).padEnd(10), 'dpr'+sz.dpr,
              '- muc tieu:', String(ts.length).padEnd(3), '| lot ra ngoai:', out.length);
  out.forEach(o => console.log('     LOT:', o.kind, 'x='+Math.round(o.x), 'y='+Math.round(o.y), 'r='+Math.round(o.r)));
  if (out.length) bad++;
  // muc tieu cham vung HUD -> khong cham duoc
  const onHud = ts.filter(o => o.y - o.r < 90 && o.x - o.r < 170);
  if (onHud.length) console.log('     canh bao: ' + onHud.length + ' muc tieu de HUD');
}
console.log(bad ? '\nFAIL: co muc tieu lot ra ngoai' : '\nPASS: khong muc tieu nao lot ra ngoai o bat ky kich thuoc nao.');
