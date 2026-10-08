(expectTtsPanel) => {
  // Read-only observation: do not disable, cancel or fast-forward UI animations.
  if (typeof expectTtsPanel !== 'boolean' || document.fonts.status !== 'loaded') return false;
  if (document.querySelector('.page-enter-active, .page-leave-active, .pop-enter-active, .pop-leave-active')) return false;
  for (const animation of document.getAnimations()) {
    if (!animation.effect || animation.pending) return false;
    const timing = animation.effect.getComputedTiming();
    // A legitimate infinite activity indicator must continue, not stall the test.
    if (timing.iterations === Infinity) continue;
    if (animation.playState !== 'finished' && animation.playState !== 'idle') return false;
  }
  const card = document.querySelector('.tts-card');
  if (!expectTtsPanel) return card === null;
  if (!card) return false;
  for (let ancestor = card; ancestor; ancestor = ancestor.parentElement) {
    const style = getComputedStyle(ancestor);
    if (style.opacity !== '1' || style.display === 'none' || style.visibility !== 'visible') return false;
  }
  const bounds = card.getBoundingClientRect();
  if (bounds.width <= 0 || bounds.height <= 0 || bounds.left < 0 || bounds.top < 0 ||
      bounds.right > innerWidth || bounds.bottom > innerHeight) return false;
  const hit = document.elementFromPoint(bounds.left + bounds.width / 2, bounds.top + bounds.height / 2);
  return hit === card || card.contains(hit);
}
