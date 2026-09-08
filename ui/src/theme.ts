import { computed, ref, watch } from 'vue';

export type Theme = 'light' | 'dark' | 'auto';
export type ResolvedTheme = 'light' | 'dark';

const STORAGE_KEY = 'yakumo-router-theme';
const MEDIA_QUERY = '(prefers-color-scheme: dark)';

function isTheme(value: string | null): value is Theme {
  return value === 'light' || value === 'dark' || value === 'auto';
}

const stored = window.localStorage.getItem(STORAGE_KEY);
const preference = ref<Theme>(isTheme(stored) ? stored : 'auto');

const media = window.matchMedia(MEDIA_QUERY);
const systemPrefersDark = ref(media.matches);

media.addEventListener('change', (event) => {
  systemPrefersDark.value = event.matches;
});

const resolvedTheme = computed<ResolvedTheme>(() => {
  if (preference.value === 'auto') {
    return systemPrefersDark.value ? 'dark' : 'light';
  }

  return preference.value;
});

function applyTheme(theme: ResolvedTheme): void {
  const root = document.documentElement;
  root.dataset.theme = theme;
  root.style.colorScheme = theme;
}

watch(resolvedTheme, applyTheme, { immediate: true });

export function useTheme() {
  function setTheme(nextTheme: Theme): void {
    preference.value = nextTheme;
    window.localStorage.setItem(STORAGE_KEY, nextTheme);
  }

  return {
    resolvedTheme,
    setTheme,
    theme: computed(() => preference.value),
  };
}
