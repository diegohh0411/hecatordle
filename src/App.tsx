import { Celebration } from "./components/Celebration";
import { GridGallery } from "./components/GridGallery";
import { Keyboard } from "./components/Keyboard";
import { ProgressHeader } from "./components/ProgressHeader";
import { StatsModal } from "./components/StatsModal";
import { useGameState } from "./hooks/useGameState";
import { useKeyboard } from "./hooks/useKeyboard";
import "./App.css";

function App() {
  const {
    status,
    errorMessage,
    store,
    currentInput,
    invalidGuess,
    solvedCount,
    guessesUsed,
    completed,
    won,
    grids,
    usedLetters,
    elapsedSeconds,
    statsOpen,
    isDarkMode,
    puzzleSource,
    handleKeyInput,
    closeStats,
    openStats,
    giveUp,
    toggleDarkMode,
    shareResult,
  } = useGameState();

  useKeyboard({ enabled: status === "ready", onInput: handleKeyInput });

  if (status === "loading") {
    return <main className="status-screen">Loading daily puzzle…</main>;
  }

  if (status === "error") {
    return <main className="status-screen">{errorMessage}</main>;
  }

  const currentGame = store?.current_game;

  return (
    <main className={`app ${invalidGuess ? "invalid" : ""}`}>
      <Celebration show={completed && won} />
      <ProgressHeader
        solvedCount={solvedCount}
        guessCount={guessesUsed}
        currentInput={currentInput}
        elapsedSeconds={elapsedSeconds}
        isDarkMode={isDarkMode}
        completed={completed}
        onToggleDarkMode={toggleDarkMode}
        onOpenStats={openStats}
        onGiveUp={giveUp}
      />

      {puzzleSource === "local" && (
        <p className="source-banner">Using local daily puzzle generator (Supabase env not configured).</p>
      )}

      <GridGallery grids={grids} />
      <Keyboard usedLetters={usedLetters} onKeyPress={handleKeyInput} disabled={completed} />

      <StatsModal
        open={statsOpen}
        stats={store!.stats}
        lastResult={
          currentGame
            ? {
                won: currentGame.won,
                guesses: currentGame.guesses.length,
                solved: solvedCount,
              }
            : null
        }
        onClose={closeStats}
        onShare={shareResult}
      />
    </main>
  );
}

export default App;
