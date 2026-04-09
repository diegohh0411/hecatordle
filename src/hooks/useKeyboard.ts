import { useEffect } from "react";

interface UseKeyboardOptions {
  enabled: boolean;
  onInput: (key: string) => void;
}

export function useKeyboard({ enabled, onInput }: UseKeyboardOptions): void {
  useEffect(() => {
    if (!enabled) {
      return;
    }

    const handleKeydown = (event: KeyboardEvent) => {
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      if (event.key === "Enter" || event.key === "Backspace" || /^[a-zA-Z]$/.test(event.key)) {
        event.preventDefault();
        onInput(event.key);
      }
    };

    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  }, [enabled, onInput]);
}
