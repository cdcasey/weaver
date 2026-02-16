import { useCallback, useRef, useState } from "react";

export function useScrollSync() {
  const [scrollTop, setScrollTop] = useState(0);
  const isSyncing = useRef(false);

  const handleScroll = useCallback((newScrollTop: number) => {
    if (isSyncing.current) return;
    isSyncing.current = true;
    setScrollTop(newScrollTop);
    requestAnimationFrame(() => {
      isSyncing.current = false;
    });
  }, []);

  return { scrollTop, handleScroll };
}
