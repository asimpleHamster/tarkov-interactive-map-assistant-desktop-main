// Translation utility
let translationMap: Map<string, string> | null = null;

// Load translations from text file
export const loadTranslations = async (): Promise<void> => {
  if (translationMap) return;

  try {
    const response = await fetch('/translations.txt');
    const text = await response.text();

    translationMap = new Map();

    const lines = text.split('\n');
    for (const line of lines) {
      const trimmed = line.trim();
      // Skip empty lines and comments
      if (!trimmed || trimmed.startsWith('#')) continue;

      const [english, chinese] = trimmed.split('=');
      if (english && chinese) {
        translationMap.set(english.trim(), chinese.trim());
      }
    }
  } catch (error) {
    console.warn('Failed to load translations:', error);
    translationMap = new Map();
  }
};

// Translate text from English to Chinese
export const translate = (text: string): string => {
  if (!translationMap) return text;
  return translationMap.get(text) || text;
};

// Initialize translations on module load
loadTranslations();
