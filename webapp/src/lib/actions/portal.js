// Shared Svelte action: teleport a node to document.body so it escapes
// `overflow: hidden` ancestors (e.g. table rows/cards) that would otherwise
// clip an absolutely/fixed-positioned popover. Mirrors the pattern used in
// webapp/src/components/features/Dropdown.svelte.
export function portal(node) {
  document.body.appendChild(node);
  return {
    destroy() {
      if (node.parentNode) {
        node.parentNode.removeChild(node);
      }
    }
  };
}
