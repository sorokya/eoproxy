import { useState, useEffect } from "react";

export default function useDebounce(fn, delay) {
  const [timer, setTimer] = useState(null);

  useEffect(() => {
    return () => {
      if (timer) {
        clearTimeout(timer);
      }
    };
  }, [timer]);

  return (...args) => {
    if (timer) {
      clearTimeout(timer);
    }
    setTimer(setTimeout(() => fn.apply(this, [...args]), delay));
  };
}
