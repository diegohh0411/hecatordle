import { describe, it, expect, beforeEach } from 'vitest';
import { 
  getOrCreatePlayerId, 
  loadLocalStore, 
  saveLocalStore, 
  initNewGame, 
  archiveCurrentGameToStats 
} from './local-store';
import { GameState, LocalStore } from '../game/types';

describe('local-store', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('generates and retrieves a player ID', () => {
    const id1 = getOrCreatePlayerId();
    expect(id1).toBeDefined();
    expect(typeof id1).toBe('string');
    
    const id2 = getOrCreatePlayerId();
    expect(id1).toBe(id2);
  });

  it('initializes a new game state', () => {
    const words = ['apple', 'beach'];
    const date = '2026-04-10';
    const game = initNewGame(words, date);
    
    expect(game.puzzleDate).toBe(date);
    expect(game.targetWords).toEqual(words);
    expect(game.guesses).toHaveLength(0);
    expect(game.solved).toEqual([false, false]);
  });

  it('saves and loads the store', () => {
    const store: LocalStore = {
      playerId: 'test-id',
      currentGame: null,
      stats: {
        gamesPlayed: 5,
        gamesWon: 3,
        currentStreak: 1,
        maxStreak: 2,
        lastPuzzleDate: '2026-04-09',
        guessDistribution: {
          '1-20': 0, '21-40': 1, '41-60': 1, '61-80': 1,
          '81-100': 0, '101-120': 0, '121-134': 0,
        }
      }
    };
    
    saveLocalStore(store);
    const loaded = loadLocalStore();
    
    // playerId is retrieved separately, but it should be set in getOrCreatePlayerId
    // which is called by loadLocalStore. 
    // Wait, let's fix the test to handle random UUIDs.
    expect(loaded.stats).toEqual(store.stats);
  });

  it('archives a won game correctly', () => {
    const game: GameState = {
      puzzleDate: '2026-04-10',
      guesses: new Array(25).fill('xxxxx'), // 25 guesses
      targetWords: ['apple'],
      solved: [true],
      completed: true,
      won: true,
    };
    
    const initialStore: LocalStore = {
      playerId: 'test-id',
      currentGame: game,
      stats: {
        gamesPlayed: 0,
        gamesWon: 0,
        currentStreak: 0,
        maxStreak: 0,
        lastPuzzleDate: null,
        guessDistribution: {
          '1-20': 0, '21-40': 0, '41-60': 0, '61-80': 0,
          '81-100': 0, '101-120': 0, '121-134': 0,
        }
      }
    };
    
    const archived = archiveCurrentGameToStats(initialStore);
    
    expect(archived.currentGame).toBeNull();
    expect(archived.stats.gamesPlayed).toBe(1);
    expect(archived.stats.gamesWon).toBe(1);
    expect(archived.stats.currentStreak).toBe(1);
    expect(archived.stats.guessDistribution['21-40']).toBe(1);
    expect(archived.stats.lastPuzzleDate).toBe('2026-04-10');
  });
});
