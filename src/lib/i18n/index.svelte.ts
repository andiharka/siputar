import { id } from './id.js';
import { en } from './en.js';
import type { Translations } from './id.js';

const catalogs: Record<string, Translations> = { id, en };

let _locale = $state<'id' | 'en'>('id');

export function getLocale() {
  return _locale;
}

export function setLocale(lang: 'id' | 'en') {
  _locale = lang;
}

export function t(): Translations {
  return catalogs[_locale] ?? id;
}
