import { useEffect } from 'react';

interface UseKeyboardProps {
  onAddLetter: (letter: string) => void;
  onRemoveLetter: () => void;
  onSubmitGuess: () => void;
  disabled?: boolean;
}

export function useKeyboard({
  onAddLetter,
  onRemoveLetter,
  onSubmitGuess,
  disabled = false,
}: UseKeyboardProps) {
  useEffect(() => {
    if (disabled) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey) return;

      if (e.key === 'Enter') {
        onSubmitGuess();
      } else if (e.key === 'Backspace') {
        onRemoveLetter();
      } else if (/^[a-zA-Z]$/.test(e.key)) {
        onAddLetter(e.key.toLowerCase());
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onAddLetter, onRemoveLetter, onSubmitGuess, disabled]);
}
