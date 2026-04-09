interface KeyboardProps {
  usedLetters: Set<string>;
  onKeyPress: (key: string) => void;
  disabled?: boolean;
}

const ROWS: string[][] = [
  ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
  ["a", "s", "d", "f", "g", "h", "j", "k", "l"],
  ["Enter", "z", "x", "c", "v", "b", "n", "m", "Backspace"],
];

export function Keyboard({ usedLetters, onKeyPress, disabled = false }: KeyboardProps) {
  return (
    <div className="keyboard" aria-label="Keyboard">
      {ROWS.map((row, rowIndex) => (
        <div className="keyboard-row" key={rowIndex}>
          {row.map((key) => {
            const lower = key.toLowerCase();
            const used = /^[a-z]$/.test(lower) && usedLetters.has(lower);
            return (
              <button
                key={key}
                type="button"
                className={`key ${used ? "used" : ""} ${key.length > 1 ? "wide" : ""}`}
                onClick={() => onKeyPress(key)}
                disabled={disabled}
              >
                {key === "Backspace" ? "⌫" : key}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}
