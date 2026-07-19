import { computed, reactive, ref } from 'vue';
import { loadConfigState } from '../api';
import type { ProviderTables, RouteTables, RouteTableState } from '../types';

type ConfigStateOptions = {
  onError: (message: string) => void;
};

export function useConfigState({ onError }: ConfigStateOptions) {
  const providers = reactive<ProviderTables>({
    openai: {},
    anthropic: {},
  });
  const routes = reactive<RouteTables>({
    openai: [],
    anthropic: [],
  });
  const routeTableState = reactive<RouteTableState>({
    active: null,
    tables: {},
  });
  const loading = ref(false);

  const totalProviders = computed(() => {
    return Object.keys(providers.openai).length + Object.keys(providers.anthropic).length;
  });
  const totalRoutes = computed(() => {
    return routes.openai.length + routes.anthropic.length;
  });
  const totalRouteTables = computed(() => {
    return Object.keys(routeTableState.tables).length;
  });

  async function loadAll(): Promise<void> {
    loading.value = true;

    try {
      const state = await loadConfigState();
      providers.openai = state.providers.openai;
      providers.anthropic = state.providers.anthropic;
      routes.openai = state.routes.openai;
      routes.anthropic = state.routes.anthropic;
      routeTableState.active = state.routeTables.active;
      routeTableState.tables = state.routeTables.tables;
    } catch (error) {
      onError(error instanceof Error ? error.message : 'Failed to load configuration.');
    } finally {
      loading.value = false;
    }
  }

  return {
    loadAll,
    loading,
    providers,
    routeTableState,
    routes,
    totalProviders,
    totalRoutes,
    totalRouteTables,
  };
}
