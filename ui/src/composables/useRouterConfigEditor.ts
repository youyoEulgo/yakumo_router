import { computed, onMounted, reactive, ref } from 'vue';
import {
  activateRouteTable as activateRouteTableApi,
  deleteProvider as deleteProviderApi,
  deleteRoute as deleteRouteApi,
  deleteRouteTable as deleteRouteTableApi,
  loadConfigState,
  saveProvider as saveProviderApi,
  saveRoute as saveRouteApi,
  saveRouteTable as saveRouteTableApi,
} from '../api';
import type {
  EditorPane,
  ProviderConfig,
  ProviderTables,
  Protocol,
  RouteRule,
  RouteTable,
  RouteTables,
  RouteTableState,
} from '../types';
import { protocolLabels } from '../types';
import { useToast } from './useToast';

export function useRouterConfigEditor() {
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

  const activeProtocol = ref<Protocol>('openai');
  const activePane = ref<EditorPane>('provider');
  const selectedProvider = ref('');
  const selectedRouteId = ref('');
  const selectedRouteTable = ref('');
  const routeTableName = ref('');
  const routeEditorOpen = ref(false);
  const draftRouteTable = reactive<RouteTable>({
    openai: [],
    anthropic: [],
  });
  const loading = ref(false);
  const savingProvider = ref(false);
  const savingRoute = ref(false);
  const savingRouteTable = ref(false);
  const activatingRouteTable = ref(false);
  const deletingProvider = ref(false);
  const deletingRouteTable = ref(false);
  const showApiKey = ref(false);
  const { clearToast, errorMessage, setError, setStatus, statusMessage, toastKey } = useToast();

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

  const selectedTable = computed(() => {
    if (selectedRouteTable.value) {
      return routeTableState.tables[selectedRouteTable.value];
    }

    return activePane.value === 'route-table' ? draftRouteTable : undefined;
  });
  const providerRoutes = computed(() => {
    return routes[activeProtocol.value].filter((route) => route.provider === selectedProvider.value);
  });
  const isEditingProvider = computed(() => Boolean(selectedProvider.value));
  const isEditingRoute = computed(() => {
    return providerRoutes.value.some((route) => route.id === routeForm.id);
  });
  const totalProviders = computed(() => {
    return Object.keys(providers.openai).length + Object.keys(providers.anthropic).length;
  });
  const totalRoutes = computed(() => {
    return routes.openai.length + routes.anthropic.length;
  });
  const totalRouteTables = computed(() => {
    return Object.keys(routeTableState.tables).length;
  });
  const topbarContext = computed(() => {
    if (activePane.value === 'route-table') {
      return selectedRouteTable.value ? `Route table / ${selectedRouteTable.value}` : 'Route table / new';
    }

    const provider = selectedProvider.value || 'new provider';
    return `${protocolLabels[activeProtocol.value]} / ${provider}`;
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

  function applyRouteTable(name: string): void {
    activePane.value = 'route-table';
    selectedRouteTable.value = name;
    routeTableName.value = name;
  }

  function resetRouteTableForm(): void {
    activePane.value = 'route-table';
    selectedRouteTable.value = '';
    routeTableName.value = '';
    draftRouteTable.openai = [];
    draftRouteTable.anthropic = [];
    clearToast();
  }

  function clearRouteTableSelection(): void {
    selectedRouteTable.value = '';
    routeTableName.value = '';
    draftRouteTable.openai = [];
    draftRouteTable.anthropic = [];
  }

  async function persistSelectedRouteTableChange(): Promise<void> {
    if (!selectedRouteTable.value || !selectedTable.value) {
      return;
    }

    savingRouteTable.value = true;
    errorMessage.value = '';

    try {
      await saveRouteTableApi(selectedRouteTable.value, selectedTable.value);
      setStatus('Route table saved.');
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to save route table.');
    } finally {
      savingRouteTable.value = false;
    }
  }

  async function toggleRouteInTable(protocol: Protocol, routeId: string, enabled: boolean): Promise<void> {
    if (!selectedTable.value) {
      return;
    }

    const ids = selectedTable.value[protocol];
    const index = ids.indexOf(routeId);
    if (enabled && index === -1) {
      ids.push(routeId);
    } else if (!enabled && index !== -1) {
      ids.splice(index, 1);
    } else {
      return;
    }

    await persistSelectedRouteTableChange();
  }

  async function moveRouteInTable(protocol: Protocol, routeId: string, direction: -1 | 1): Promise<void> {
    if (!selectedTable.value) {
      return;
    }

    const ids = selectedTable.value[protocol];
    const index = ids.indexOf(routeId);
    const nextIndex = index + direction;
    if (index === -1 || nextIndex < 0 || nextIndex >= ids.length) {
      return;
    }

    ids.splice(index, 1);
    ids.splice(nextIndex, 0, routeId);
    await persistSelectedRouteTableChange();
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

  async function loadAll(): Promise<void> {
    loading.value = true;
    errorMessage.value = '';

    try {
      const state = await loadConfigState();
      providers.openai = state.providers.openai;
      providers.anthropic = state.providers.anthropic;
      routes.openai = state.routes.openai;
      routes.anthropic = state.routes.anthropic;
      routeTableState.active = state.routeTables.active;
      routeTableState.tables = state.routeTables.tables;
      if (selectedRouteTable.value && !routeTableState.tables[selectedRouteTable.value]) {
        clearRouteTableSelection();
      }
      if (!selectedRouteTable.value && routeTableState.active) {
        applyRouteTable(routeTableState.active);
      }
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to load configuration.');
    } finally {
      loading.value = false;
    }
  }

  async function saveProvider(): Promise<void> {
    savingProvider.value = true;
    errorMessage.value = '';

    try {
      const name = providerForm.name.trim();
      const result = await saveProviderApi(activeProtocol.value, name, {
        base_url: providerForm.base_url,
        api_key: providerForm.api_key,
      });
      await loadAll();
      applyProvider(result.name, result.provider);
      setStatus(result.updated ? 'Provider saved.' : 'Provider created.');
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to save provider.');
    } finally {
      savingProvider.value = false;
    }
  }

  async function deleteSelectedProvider(): Promise<void> {
    if (!selectedProvider.value) {
      return;
    }

    deletingProvider.value = true;
    errorMessage.value = '';

    try {
      const result = await deleteProviderApi(activeProtocol.value, selectedProvider.value);
      await loadAll();
      resetProviderForm();
      setStatus(`Provider deleted. ${result.removed_routes} route(s) removed.`);
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to delete provider.');
    } finally {
      deletingProvider.value = false;
    }
  }

  async function saveRoute(): Promise<void> {
    savingRoute.value = true;
    errorMessage.value = '';
    routeForm.provider = selectedProvider.value;

    try {
      const result = await saveRouteApi(activeProtocol.value, routeForm);
      await loadAll();
      applyRoute(result.route);
      setStatus(result.updated ? 'Rule saved.' : 'Rule created.');
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to save rule.');
    } finally {
      savingRoute.value = false;
    }
  }

  async function deleteSelectedRoute(): Promise<void> {
    if (!selectedRouteId.value) {
      return;
    }

    savingRoute.value = true;
    errorMessage.value = '';

    try {
      const result = await deleteRouteApi(activeProtocol.value, selectedRouteId.value);
      await loadAll();
      resetRouteForm();
      setStatus(`Rule ${result.id} deleted.`);
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to delete rule.');
    } finally {
      savingRoute.value = false;
    }
  }

  async function saveRouteTable(): Promise<void> {
    const name = routeTableName.value.trim();
    if (!name) {
      setError('Route table name is required.');
      return;
    }

    savingRouteTable.value = true;
    errorMessage.value = '';

    try {
      await saveRouteTableApi(name, selectedTable.value ?? { openai: [], anthropic: [] });
      selectedRouteTable.value = name;
      await loadAll();
      applyRouteTable(name);
      setStatus('Route table saved.');
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to save route table.');
    } finally {
      savingRouteTable.value = false;
    }
  }

  async function deleteSelectedRouteTable(): Promise<void> {
    if (!selectedRouteTable.value) {
      return;
    }

    deletingRouteTable.value = true;
    errorMessage.value = '';

    try {
      const deleted = selectedRouteTable.value;
      await deleteRouteTableApi(deleted);
      await loadAll();
      clearRouteTableSelection();
      setStatus(`Route table ${deleted} deleted.`);
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to delete route table.');
    } finally {
      deletingRouteTable.value = false;
    }
  }

  async function activateRouteTable(): Promise<void> {
    if (!selectedRouteTable.value) {
      return;
    }

    activatingRouteTable.value = true;
    errorMessage.value = '';

    try {
      await activateRouteTableApi(selectedRouteTable.value);
      routeTableState.active = selectedRouteTable.value;
      setStatus('Route table activated.');
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to activate route table.');
    } finally {
      activatingRouteTable.value = false;
    }
  }

  onMounted(() => {
    void loadAll();
  });

  return {
    activateRouteTable,
    activatingRouteTable,
    activePane,
    activeProtocol,
    deleteSelectedProvider,
    deleteSelectedRoute,
    deleteSelectedRouteTable,
    deletingProvider,
    deletingRouteTable,
    errorMessage,
    isEditingProvider,
    isEditingRoute,
    loadAll,
    loading,
    moveRouteInTable,
    providerForm,
    providerRoutes,
    providers,
    routeEditorOpen,
    routeForm,
    routeTableName,
    routeTableState,
    routes,
    saveProvider,
    saveRoute,
    saveRouteTable,
    savingProvider,
    savingRoute,
    savingRouteTable,
    selectNewProvider,
    selectProvider,
    selectedProvider,
    selectedRouteId,
    selectedRouteTable,
    selectedTable,
    showApiKey,
    startNewRoute,
    statusMessage,
    toastKey,
    toggleRouteInTable,
    topbarContext,
    totalProviders,
    totalRoutes,
    totalRouteTables,
    updateProviderField,
    updateRouteField,
    resetRouteTableForm,
    applyRouteTable,
    applyRoute,
  };
}
