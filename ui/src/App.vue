<script setup lang="ts">
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
} from './api';
import ProviderEditor from './components/ProviderEditor.vue';
import RouteTableEditor from './components/RouteTableEditor.vue';
import SidebarNav from './components/SidebarNav.vue';
import StatusBar from './components/StatusBar.vue';
import type {
  EditorPane,
  ProviderConfig,
  ProviderTables,
  Protocol,
  RouteRule,
  RouteTables,
  RouteTable,
  RouteTableState,
} from './types';

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
const statusMessage = ref('');
const errorMessage = ref('');

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

function setStatus(message: string): void {
  statusMessage.value = message;
  errorMessage.value = '';
}

function setError(message: string): void {
  errorMessage.value = message;
  statusMessage.value = '';
}

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
  statusMessage.value = '';
  errorMessage.value = '';
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
  statusMessage.value = '';
  errorMessage.value = '';
}

function clearRouteTableSelection(): void {
  selectedRouteTable.value = '';
  routeTableName.value = '';
  draftRouteTable.openai = [];
  draftRouteTable.anthropic = [];
}

function toggleRouteInTable(protocol: Protocol, routeId: string, enabled: boolean): void {
  if (!selectedTable.value) {
    return;
  }

  const ids = selectedTable.value[protocol];
  const index = ids.indexOf(routeId);
  if (enabled && index === -1) {
    ids.push(routeId);
  } else if (!enabled && index !== -1) {
    ids.splice(index, 1);
  }
}

function moveRouteInTable(protocol: Protocol, routeId: string, direction: -1 | 1): void {
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
}

function applyRoute(route: RouteRule): void {
  routeForm.id = route.id;
  routeForm.match = route.match;
  routeForm.match_type = route.match_type ?? 'contains';
  routeForm.provider = route.provider;
  routeForm.model = route.model;
  routeForm.forward_only = route.forward_only ?? false;
  selectedRouteId.value = route.id;
}

function resetRouteForm(): void {
  routeForm.id = '';
  routeForm.match = '';
  routeForm.match_type = 'contains';
  routeForm.provider = selectedProvider.value;
  routeForm.model = '';
  routeForm.forward_only = false;
  selectedRouteId.value = '';
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
</script>

<template>
  <main class="app-shell">
    <header class="topbar">
      <div>
        <h1>Yakumo Switch</h1>
        <p>Manage providers and their model routing rules.</p>
      </div>
      <button class="ghost-button" type="button" :disabled="loading" @click="loadAll">
        Refresh
      </button>
    </header>

    <section class="workspace">
      <SidebarNav :active-pane="activePane" :active-protocol="activeProtocol" :loading="loading" :providers="providers"
        :routes="routes" :route-tables="routeTableState" :selected-provider="selectedProvider"
        :selected-route-table="selectedRouteTable" @new-provider="selectNewProvider" @select-provider="selectProvider"
        @new-route-table="resetRouteTableForm" @select-route-table="applyRouteTable" />

      <section class="editor">
        <RouteTableEditor v-if="activePane === 'route-table'" v-model:route-table-name="routeTableName"
          :activating="activatingRouteTable" :active-route-table="routeTableState.active" :deleting="deletingRouteTable"
          :route-table="selectedTable" :routes="routes" :saving="savingRouteTable"
          :selected-route-table="selectedRouteTable" @activate="activateRouteTable" @delete="deleteSelectedRouteTable"
          @save="saveRouteTable" @toggle-route="toggleRouteInTable" @move-route="moveRouteInTable" />

        <ProviderEditor v-else v-model:show-api-key="showApiKey" :active-protocol="activeProtocol"
          :deleting-provider="deletingProvider" :is-editing-provider="isEditingProvider"
          :is-editing-route="isEditingRoute" :provider-form="providerForm" :provider-routes="providerRoutes"
          :route-form="routeForm" :saving-provider="savingProvider" :saving-route="savingRoute"
          :selected-provider="selectedProvider" :selected-route-id="selectedRouteId" @save-provider="saveProvider"
          @update-provider-field="updateProviderField" @update-route-field="updateRouteField"
          @delete-provider="deleteSelectedProvider" @reset-route="resetRouteForm" @select-route="applyRoute"
          @save-route="saveRoute" @delete-route="deleteSelectedRoute" />

        <StatusBar :error="errorMessage" :status="statusMessage" />
      </section>
    </section>
  </main>
</template>
