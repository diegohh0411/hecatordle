import React from 'react';

const ROWS = [
  ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
  ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
  ['Enter', 'z', 'x', 'c', 'v', 'b', 'n', 'm', 'Back']
];

interface KeyboardProps {
  onAddLetter: (letter: string) => void;
  onRemoveLetter: () => void;
  onSubmitGuess: () => void;
  usedLetters: Set<string>;
}

export const Keyboard: React.FC<KeyboardProps> = ({
  onAddLetter,
  onRemoveLetter,
  onSubmitGuess,
  usedLetters
}) => {
  return (
    <div className="keyboard">
      {ROWS.map((row, i) => (
        <div key={i} className="keyboard-row">
          {row.map(key => {
            let label = key;
            let onClick = () => onAddLetter(key);
            let extraClass = "";

            if (key === 'Enter') {
              onClick = onSubmitGuess;
              extraClass = "wide-key";
            } else if (key === 'Back') {
              label = "⌫";
              onClick = onRemoveLetter;
              extraClass = "wide-key";
            } else {
              if (usedLetters.has(key)) {
                extraClass = "used";
              }
            }

            return (
              <button
                key={key}
                className={`key ${extraClass}`}
                onClick={(e) => {
                  e.currentTarget.blur();
                  onClick();
                }}
              >
                {label}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
};
