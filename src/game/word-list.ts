import wordBankData from '../../tools/word-frequency/word_bank.json';

export const WORD_BANK: string[] = wordBankData.map((entry: { word: string }) => entry.word);
