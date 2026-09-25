/* Render that: chup tat ca man hinh + 6 ban do de kiem tra bang mat. */
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const { createCanvas } = require('canvas');

const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const code = html.match(/<script>([\s\S]*?)<\/script>/)[1];

const W = +(process.env.GW || 1280), H = +(process.env.GH || 720), DPR = 1;
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
vm.runInContext(code, s, { timeout: 15000 });

const step = (n = 1, ms = 16) => { for (let i = 0; i < n; i++) { clock.t += ms; const c = rafCbs.splice(0); for (const cb of c) cb(clock.t); } };
const PFX = process.env.PFX || 'd';
const shot = (n) => { fs.writeFileSync(path.join(__dirname, 'v-' + PFX + '-' + n + '.png'), canvas.toBuffer('image/png')); };
const TAG = process.env.TAG || '';

// MENU
s.S.coins = 2450; s.S.best = 18400; s.S.streak = 3; s.S.plays = 27;
step(20); shot('1-menu');

// 6 BAN DO trong luc choi
for (let i = 0; i < s.MAPS.length; i++) {
  s.S.map = s.MAPS[i].id; s.S.maps = s.MAPS.map(m => m.id);
  s.S.cats = ['gold','pink','blue']; s.S.ups = { tap:4, coin:3, time:2, luck:3, combo:2, crit:1, auto:1 };
  s.startRound();
  // toc do nguoi that (~2.5 lan/giay) de anh phan anh luc choi
  for (let f = 0; f < 90; f++) {
    if (f % 26 === 0) {
      const vis = s.targets.filter(o => o.y > s.PLAY_TOP + 10);
      if (vis.length) s.tap(vis[vis.length - 1].x, vis[vis.length - 1].y);
    }
    step(1);
  }
  step(2);
  shot('2-map-' + s.MAPS[i].id);
}

// HET VAN
s.timeLeft = 0.001; step(4); shot('3-over');
// CUA HANG - nang cap
s.S.coins = 4200; s.S.ups = { tap:5, coin:3, time:2, luck:2, combo:2 };
s.st = 3; s.shopTab = 0; s.scrollY = 0; step(8); shot('4-shop-up');
s.scrollY = 170; step(4); shot('4b-shop-up2');

// CUA HANG - meo
s.S.cats = ['gold','pink','blue'];
s.st = 3; s.shopTab = 1; s.scrollY = 0; step(6); shot('5-shop-cats');

// CHON BAN DO
s.st = 6; s.pickIdx = 3; s.S.coins = 4200; step(6); shot('6-pick');

// THANH TICH  (ST_MENU=0 ST_PLAY=1 ST_OVER=2 ST_SHOP=3 ST_STATS=4 ST_PICK=5)
s.S.total = 128400; s.S.plays = 27; s.S.best = 18400;
s.st = 4; step(6); shot('7-stats');

console.log('da render xong (' + PFX + '), cac anh:');
fs.readdirSync(__dirname).filter(f => f.startsWith('v-' + PFX + '-'))
  .forEach(f => console.log('  ' + f + '  ' + (fs.statSync(path.join(__dirname, f)).size / 1024).toFixed(0) + 'KB'));
