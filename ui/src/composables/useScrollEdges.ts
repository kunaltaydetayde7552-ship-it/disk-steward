// Tracks whether a scroll box has moved from its top and whether more lies below its fold, so the box can
// draw a hairline under its head and fade its lower edge instead of slicing a line of text in half.
// The box's first child is observed too: its height changes when the content does.
import { onBeforeUnmount, onMounted, ref, type Ref } from "vue";

export function useScrollEdges(box: Ref<HTMLElement | null>) {
  const scrolled = ref(false);
  const more = ref(false);

  function measure() {
    const b = box.value;
    if (!b) return;
    scrolled.value = b.scrollTop > 2;
    more.value = b.scrollHeight - b.scrollTop - b.clientHeight > 4;
  }

  let ro: ResizeObserver | null = null;
  onMounted(() => {
    measure();
    if (typeof ResizeObserver === "undefined" || !box.value) return;
    ro = new ResizeObserver(measure);
    ro.observe(box.value);
    if (box.value.firstElementChild) ro.observe(box.value.firstElementChild);
  });
  onBeforeUnmount(() => ro?.disconnect());

  return { scrolled, more, measure };
}
