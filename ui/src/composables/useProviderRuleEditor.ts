import { computed, reactive, ref } from 'vue';
import {
  deleteProvider as deleteProviderApi,
  deleteRoute as deleteRouteApi,
  saveProvider as saveProviderApi,
  saveRoute as saveRouteApi,
} from '../api';
import type { EditorPane, ProviderConfig, Protocol, RouteRule, RouteTables } from '../types';

type ProviderRuleEditorOptions = {
  activePane: { value: EditorPane };
  routes: RouteTables;
  reload: () => Promise<void>;
  clearToast: () => void;
  onError: (message: string) => void;
  onStatus: (message: string) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
};

export function useProviderRuleEditor({
  activePane,
  clearToast,
  onError,
  onStatus,
  reload,
  routes,
  t,
}: ProviderRuleEditorOptions) {
  const activeProtocol = ref<Protocol>('openai');
  const selectedProvider = ref('');
  const selectedRouteId = ref('');
  const routeEditorOpen = ref(false);
  const savingProvider = ref(false);
  const savingRoute = ref(false);
  const deletingProvider = ref(false);
  const showApiKey = ref(false);

  const providerForm = reactive({
    name: '',
    base_url: '',
    api_key: '',
  });

  const routeForm = reactive<RouteRule>({
    id: '',
    match: '',
    match_type: 'contains',
    provider: '',
    model: '',
    forward_only: false,
  });

  const providerRoutes = computed(() => {
    return routes[activeProtocol.value].filter(
      (route) => route.provider === selectedProvider.value,
    );
  });
  const isEditingProvider = computed(() => Boolean(selectedProvider.value));
  const isEditingRoute = computed(() => {
    return providerRoutes.value.some((route) => route.id === routeForm.id);
  });

  function applyProvider(name: string, provider: ProviderConfig): void {
    activePane.value = 'provider';
    selectedProvider.value = name;
    providerForm.name = name;
    providerForm.base_url = provider.base_url;
    providerForm.api_key = provider.api_key;
    resetRouteForm();
  }

  function resetProviderForm(): void {
    activePane.value = 'provider';
    selectedProvider.value = '';
    providerForm.name = '';
    providerForm.base_url = '';
    providerForm.api_key = '';
    resetRouteForm();
    clearToast();
  }

  function selectNewProvider(protocol: Protocol): void {
    activeProtocol.value = protocol;
    resetProviderForm();
  }

  function selectProvider(protocol: Protocol, name: string, provider: ProviderConfig): void {
    activeProtocol.value = protocol;
    applyProvider(name, provider);
  }

  function applyRoute(route: RouteRule): void {
    routeForm.id = route.id;
    routeForm.match = route.match;
    routeForm.match_type = route.match_type ?? 'contains';
    routeForm.provider = route.provider;
    routeForm.model = route.model;
    routeForm.forward_only = route.forward_only ?? false;
    selectedRouteId.value = route.id;
    routeEditorOpen.value = true;
  }

  function resetRouteForm(): void {
    routeForm.id = '';
    routeForm.match = '';
    routeForm.match_type = 'contains';
    routeForm.provider = selectedProvider.value;
    routeForm.model = '';
    routeForm.forward_only = false;
    selectedRouteId.value = '';
    routeEditorOpen.value = false;
  }

  function startNewRoute(): void {
    resetRouteForm();
    routeEditorOpen.value = true;
  }

  function updateProviderField(field: 'name' | 'base_url' | 'api_key', value: string): void {
    providerForm[field] = value;
  }

  function updateRouteField(field: keyof RouteRule, value: boolean | string): void {
    switch (field) {
      case 'forward_only':
        routeForm.forward_only = Boolean(value);
        break;
      case 'match_type':
        routeForm.match_type = value as RouteRule['match_type'];
        break;
      case 'id':
      case 'match':
      case 'provider':
      case 'model':
        routeForm[field] = String(value);
        break;
    }
  }

  async function saveProvider(): Promise<void> {
    savingProvider.value = true;

    try {
      const name = providerForm.name.trim();
      const result = await saveProviderApi(activeProtocol.value, name, {
        base_url: providerForm.base_url,
        api_key: providerForm.api_key,
      });
      await reload();
      applyProvider(result.name, result.provider);
      onStatus(result.updated ? t('providerSaved') : t('providerCreated'));
    } catch (error) {
      onError(error instanceof Error ? error.message : t('failedSaveProvider'));
    } finally {
      savingProvider.value = false;
    }
  }

  async function deleteSelectedProvider(): Promise<void> {
    if (!selectedProvider.value) {
      return;
    }

    deletingProvider.value = true;

    try {
      const result = await deleteProviderApi(activeProtocol.value, selectedProvider.value);
      await reload();
      resetProviderForm();
      onStatus(t('providerDeleted', { count: result.removed_routes }));
    } catch (error) {
      onError(error instanceof Error ? error.message : t('failedDeleteProvider'));
    } finally {
      deletingProvider.value = false;
    }
  }

  async function saveRoute(): Promise<void> {
    savingRoute.value = true;
    routeForm.provider = selectedProvider.value;

    try {
      const result = await saveRouteApi(activeProtocol.value, routeForm);
      await reload();
      applyRoute(result.route);
      onStatus(result.updated ? t('ruleSaved') : t('ruleCreated'));
    } catch (error) {
      onError(error instanceof Error ? error.message : t('failedSaveRule'));
    } finally {
      savingRoute.value = false;
    }
  }

  async function deleteSelectedRoute(): Promise<void> {
    if (!selectedRouteId.value) {
      return;
    }

    savingRoute.value = true;

    try {
      const result = await deleteRouteApi(activeProtocol.value, selectedRouteId.value);
      await reload();
      resetRouteForm();
      onStatus(t('routeDeleted', { id: result.id }));
    } catch (error) {
      onError(error instanceof Error ? error.message : t('failedDeleteRule'));
    } finally {
      savingRoute.value = false;
    }
  }

  return {
    activeProtocol,
    applyRoute,
    deleteSelectedProvider,
    deleteSelectedRoute,
    deletingProvider,
    isEditingProvider,
    isEditingRoute,
    providerForm,
    providerRoutes,
    routeEditorOpen,
    routeForm,
    saveProvider,
    saveRoute,
    savingProvider,
    savingRoute,
    selectNewProvider,
    selectProvider,
    selectedProvider,
    selectedRouteId,
    showApiKey,
    startNewRoute,
    updateProviderField,
    updateRouteField,
  };
}
