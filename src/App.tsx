import { useGameState } from './hooks/useGameState';
import { useKeyboard } from './hooks/useKeyboard';
import { ProgressHeader } from './components/ProgressHeader';
import { GridGallery } from './components/GridGallery';
import { Keyboard } from './components/Keyboard';
import { StatsModal } from './components/StatsModal';
import { Celebration } from './components/Celebration';
import { getKeyboardUsedLetters } from './game/guess-validation';
import { useState, useEffect } from 'react';
import './App.css';

function App() {
  const {
    gameState,
    stats,
    currentGuess,
    error,
    addLetter,
    removeLetter,
    submitGuess,
  } = useGameState();

  const [isStatsOpen, setIsStatsOpen] = useState(false);

  useEffect(() => {
    if (gameState?.completed) {
      setTimeout(() => setIsStatsOpen(true), 2000);
    }
  }, [gameState?.completed]);

  useKeyboard({
    onAddLetter: addLetter,
    onRemoveLetter: removeLetter,
    onSubmitGuess: submitGuess,
    disabled: gameState?.completed || isStatsOpen,
  });

  if (!gameState) {
    return <div className="loading">Loading puzzle...</div>;
  }

  const solvedCount = gameState.solved.filter(s => s).length;
  const usedLetters = getKeyboardUsedLetters(gameState.guesses);

  return (
    <>
      <ProgressHeader
        solvedCount={solvedCount}
        totalWords={gameState.targetWords.length}
        guessCount={gameState.guesses.length}
        maxGuesses={134}
        completed={gameState.completed}
      />

      <div className="header-actions">
        <button className="icon-button" onClick={() => setIsStatsOpen(true)}>📈</button>
      </div>
      
      <GridGallery
        targetWords={gameState.targetWords}
        guesses={gameState.guesses}
        currentGuess={currentGuess}
        solved={gameState.solved}
      />

      {error && <div className="error-toast" key={error}>{error}</div>}

      <Celebration show={gameState.won && gameState.completed} />

      <StatsModal
        isOpen={isStatsOpen}
        onClose={() => setIsStatsOpen(false)}
        stats={stats}
      />

      <Keyboard
        onAddLetter={addLetter}
        onRemoveLetter={removeLetter}
        onSubmitGuess={submitGuess}
        usedLetters={usedLetters}
      />
    </>
  );
}

export default App;
