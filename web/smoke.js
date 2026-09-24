/* Smoke test game web: chay script trong Node voi canvas/DOM gia.
   Bat moi loi runtime + kiem tra logic co ban. */
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const m = html.match(/<script>([\s\S]*?)<\/script>/);
if (!m) { console.error('FAIL: khong tim thay <script>'); process.exit(1); }
const code = m[1];

// ---- canvas gia: ghi lai moi lenh ve de kiem tra co that su ve khong ----
const calls = {};
function rec(name) { calls[name] = (calls[name] || 0) + 1; }
function makeCtx() {
  const noop = () => {};
  const ctx = new Proxy({}, {
    get(t, prop) {
      if (prop === 'canvas') return { width: 1280, height: 720 };
      if (prop === 'createLinearGradient')
        return () => ({ addColorStop: noop });
      if (prop === 'measureText') return () => ({ width: 10 });
      if (prop in t) return t[prop];
      // gan gia: ham ve co ghi nhan
      return (...a) => rec(String(prop));
    },
    set(t, prop, v) { t[prop] = v; return true; }
  });
  return ctx;
}

let rafCbs = [];
const sandbox = {
  console,
  performance: { now: () => Date.now() },
  requestAnimationFrame: (cb) => { rafCbs.push(cb); return rafCbs.length; },
  setTimeout: (fn) => 0,
  localStorage: { getItem: () => null, setItem: () => {} },
  document: {
    getElementById: () => ({ getContext: makeCtx, addEventListener: () => {}, style: {} }),
    addEventListener: () => {},
    hidden: false
  },
  window: null,
  innerWidth: 1280,
  innerHeight: 720,
  devicePixelRatio: 1,
  AudioContext: function () {
    return {
      state: 'running', currentTime: 0, destination: {},
      resume: () => {},
      createOscillator: () => ({ type: '', frequency: { setValueAtTime: noop2, exponentialRampToValueAtTime: noop2 }, connect: noop2, start: noop2, stop: noop2 }),
      createGain: () => ({ gain: { setValueAtTime: noop2, exponentialRampToValueAtTime: noop2 }, connect: noop2 })
    };
  },
  Math, Date, JSON, parseInt, isNaN
};
function noop2() {}
sandbox.window = sandbox;
sandbox.globalThis = sandbox;
sandbox.window.addEventListener = () => {};
sandbox.window.PointerEvent = function () {};

const errors = [];
try {
  vm.createContext(sandbox);
  vm.runInContext(code, sandbox, { timeout: 10000 });
} catch (e) {
  errors.push('LOAD: ' + e.message);
}

function fail(msg) { errors.push(msg); }

// ---- kiem tra cac thanh phan co ton tai ----
for (const name of ['startGame', 'pointAt', 'loop', 'spawnTargets', 'overGame', 'drawTarget']) {
  if (typeof sandbox[name] !== 'function') fail('thieu ham: ' + name);
}
if (!sandbox.SFX || typeof sandbox.SFX.hit !== 'function') fail('thieu bo SFX');

// ---- chay vong choi: 600 frame, bat dau + cham muc tieu ----
try {
  if (sandbox.startGame) {
    sandbox.startGame();
    for (let i = 0; i < 600; i++) {
      // chon 1 muc tieu de cham
      if (sandbox.targets && sandbox.targets.length) {
        const t = sandbox.targets[0];
        sandbox.pointAt(t.x, t.y);
      }
      const cbs = rafCbs.splice(0);
      for (const cb of cbs) cb(Date.now());
    }
  }
} catch (e) {
  fail('LOOP: ' + e.message + '\n' + (e.stack || '').split('\n').slice(0, 3).join('\n'));
}

// ---- chay het gio -> het mang ----
try {
  if (sandbox.startGame) {
    sandbox.startGame();
    sandbox.timeLeft = 0.001;
    const cbs = rafCbs.splice(0);
    for (const cb of cbs) cb(Date.now() + 100);
  }
} catch (e) { fail('OVER: ' + e.message); }

// ---- ket qua ----
const drew = (calls.fill || 0) + (calls.arc || 0) + (calls.stroke || 0) + (calls.fillRect || 0);
console.log('--- KET QUA ---');
console.log('lenh ve da phat ra:', drew);
console.log('fillRect calls   :', calls.fillRect || 0);
if (drew < 100) fail('ve qua it — co the canvas khong hoat dong');
if ((calls.fillRect || 0) < 10) fail('khong ve nen');

if (errors.length) {
  console.log('\nLOI (' + errors.length + '):');
  errors.forEach(e => console.log(' - ' + e));
  process.exit(1);
}
console.log('\nPASS: khong loi runtime, game ve binh thuong.');
