// Generated DOM/animation doubles only; these are not rendered browser acceptance.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import vm from 'node:vm';

const source = readFileSync(new URL('./ui-screenshot-readiness.js', import.meta.url), 'utf8');
function observe(options = {}, expects = true) {
  const child = {};
  const parent = { parentElement: null };
  const card = {
    parentElement: parent,
    getBoundingClientRect: () => options.bounds ?? { left: 20, top: 20, right: 340, bottom: 500, width: 320, height: 480 },
    contains: node => node === child,
  };
  const document = {
    fonts: { status: options.fontStatus ?? 'loaded' },
    querySelector: selector => selector === '.tts-card' ? (options.cardMissing ? null : card) : (options.transition ? {} : null),
    getAnimations: () => options.animations ?? [],
    elementFromPoint: () => options.occluded ? {} : child,
  };
  const environment = { document, innerWidth: 1280, innerHeight: 720,
    getComputedStyle: element => ({
      opacity: element === card ? options.cardOpacity ?? '1' : options.parentOpacity ?? '1',
      display: options.display ?? 'block', visibility: options.visibility ?? 'visible',
    }),
  };
  return vm.runInNewContext(source, environment)(expects);
}
const animation = (playState, { pending = false, iterations = 1 } = {}) => ({
  pending, playState, effect: { getComputedTiming: () => ({ iterations }) },
});

test('settled visible card is accepted', () => assert.equal(observe(), true));
test('font load is required', () => assert.equal(observe({ fontStatus: 'loading' }), false));
test('Vue enter and leave classes are not a stable screenshot', () => assert.equal(observe({ transition: true }), false));
test('finite animation running is rejected', () => assert.equal(observe({ animations: [animation('running')] }), false));
test('finite paused midpoint is rejected', () => assert.equal(observe({ animations: [animation('paused')] }), false));
test('pending even with finished playState is rejected', () => assert.equal(observe({ animations: [animation('finished', { pending: true })] }), false));
test('settled finite animation can remain in document animation list', () => assert.equal(observe({ animations: [animation('finished')] }), true));
test('idle finite animation is accepted', () => assert.equal(observe({ animations: [animation('idle')] }), true));
test('legitimate infinite activity indicator is not cancelled or awaited forever', () => assert.equal(observe({ animations: [animation('running', { iterations: Infinity })] }), true));
test('missing animation effect is rejected', () => assert.equal(observe({ animations: [{ pending: false, effect: null, playState: 'running' }] }), false));
test('card fade must be complete', () => assert.equal(observe({ cardOpacity: '0.25' }), false));
test('ancestor fade must be complete', () => assert.equal(observe({ parentOpacity: '0.25' }), false));
test('hidden card cannot become visual evidence', () => assert.equal(observe({ visibility: 'hidden' }), false));
test('display none is rejected', () => assert.equal(observe({ display: 'none' }), false));
test('clipped card is rejected', () => assert.equal(observe({ bounds: { left: 20, top: -20, right: 340, bottom: 460, width: 320, height: 480 } }), false));
test('occluded center is rejected', () => assert.equal(observe({ occluded: true }), false));
test('card required for playing and returned scenarios', () => assert.equal(observe({ cardMissing: true }), false));
test('settings scenario expects the card absent', () => {
  assert.equal(observe({ cardMissing: true }, false), true);
  assert.equal(observe({}, false), false);
});
test('scene type cannot be silently coerced', () => assert.equal(observe({}, 'true'), false));

test('all screenshot callers execute the readiness guard before capture', () => {
  const java = readFileSync(new URL('../java/com/medwarp/reader/browserpoc/Vue3PreviewTtsCacheIsolationTest.java', import.meta.url), 'utf8');
  const start = java.indexOf('private static void screenshot(');
  const end = java.indexOf('@SuppressWarnings', start);
  const method = java.slice(start, end);
  assert.match(method, /ui-screenshot-readiness\.js/);
  assert.match(method, /page\.waitForFunction\(ready, expectTtsPanel\)/);
  assert.ok(method.indexOf('page.waitForFunction') < method.indexOf('page.screenshot'));
  assert.match(method, /Expected generated playback must still be active/);
});
