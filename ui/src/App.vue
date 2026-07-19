<script setup lang="ts">
import ProviderEditor from './components/ProviderEditor.vue';
import RouteTableEditor from './components/RouteTableEditor.vue';
import SidebarNav from './components/SidebarNav.vue';
import StatusBar from './components/StatusBar.vue';
import { useRouterConfigEditor } from './composables/useRouterConfigEditor';

const {
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
} = useRouterConfigEditor();
</script>

<template>
  <main class="app-shell">
    <header class="topbar">
      <div class="topbar-title">
        <span class="app-mark" aria-hidden="true">Y</span>
        <div>
          <h1>Yakumo Router</h1>
          <p>{{ topbarContext }}</p>
        </div>
      </div>
      <div class="topbar-actions">
        <div class="topbar-stats" aria-label="Configuration summary">
          <span class="stat-pill active-route-pill">
            <span class="stat-label">Active</span>
            <strong>{{ routeTableState.active ?? 'None' }}</strong>
          </span>
          <span class="stat-pill">
            <strong>{{ totalProviders }}</strong>
            <span class="stat-label">Providers</span>
          </span>
          <span class="stat-pill">
            <strong>{{ totalRoutes }}</strong>
            <span class="stat-label">Rules</span>
          </span>
          <span class="stat-pill">
            <strong>{{ totalRouteTables }}</strong>
            <span class="stat-label">Tables</span>
          </span>
        </div>
        <button
          class="icon-button refresh-button"
          type="button"
          :disabled="loading"
          :aria-label="loading ? 'Refreshing' : 'Refresh'"
          :title="loading ? 'Refreshing' : 'Refresh'"
          @click="loadAll"
        >
          <svg
            class="button-icon"
            :class="{ spinning: loading }"
            aria-hidden="true"
            viewBox="0 0 24 24"
            fill="none"
          >
            <path
              d="M20 11a8 8 0 0 0-14.7-4.4L4 8"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <path
              d="M4 4v4h4"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <path
              d="M4 13a8 8 0 0 0 14.7 4.4L20 16"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <path
              d="M20 20v-4h-4"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      </div>
    </header>

    <section class="workspace">
      <SidebarNav
        :active-pane="activePane"
        :active-protocol="activeProtocol"
        :loading="loading"
        :providers="providers"
        :routes="routes"
        :route-tables="routeTableState"
        :selected-provider="selectedProvider"
        :selected-route-table="selectedRouteTable"
        @new-provider="selectNewProvider"
        @select-provider="selectProvider"
        @new-route-table="resetRouteTableForm"
        @select-route-table="applyRouteTable"
      />

      <section class="editor">
        <RouteTableEditor
          v-if="activePane === 'route-table'"
          v-model:route-table-name="routeTableName"
          :activating="activatingRouteTable"
          :active-route-table="routeTableState.active"
          :deleting="deletingRouteTable"
          :route-table="selectedTable"
          :routes="routes"
          :saving="savingRouteTable"
          :selected-route-table="selectedRouteTable"
          @activate="activateRouteTable"
          @delete="deleteSelectedRouteTable"
          @save="saveRouteTable"
          @toggle-route="toggleRouteInTable"
          @move-route="moveRouteInTable"
        />

        <ProviderEditor
          v-else
          v-model:show-api-key="showApiKey"
          :active-protocol="activeProtocol"
          :deleting-provider="deletingProvider"
          :is-editing-provider="isEditingProvider"
          :is-editing-route="isEditingRoute"
          :provider-form="providerForm"
          :provider-routes="providerRoutes"
          :route-editor-open="routeEditorOpen"
          :route-form="routeForm"
          :saving-provider="savingProvider"
          :saving-route="savingRoute"
          :selected-provider="selectedProvider"
          :selected-route-id="selectedRouteId"
          @save-provider="saveProvider"
          @update-provider-field="updateProviderField"
          @update-route-field="updateRouteField"
          @delete-provider="deleteSelectedProvider"
          @reset-route="startNewRoute"
          @select-route="applyRoute"
          @save-route="saveRoute"
          @delete-route="deleteSelectedRoute"
        />

        <StatusBar :key="toastKey" :error="errorMessage" :status="statusMessage" />
      </section>
    </section>
  </main>
</template>
