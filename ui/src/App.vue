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

<style scoped>
.app-shell {
  min-height: 100vh;
}

.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  min-height: 84px;
  padding: 20px 32px;
  border-bottom: 1px solid rgba(195, 204, 217, 0.82);
  background: rgba(255, 255, 255, 0.78);
  backdrop-filter: blur(14px);
}

.topbar-title,
.topbar-actions,
.topbar-stats,
.stat-pill {
  display: flex;
  align-items: center;
}

.topbar-title {
  min-width: 0;
  gap: 12px;
}

.topbar-actions {
  justify-content: flex-end;
  gap: 12px;
  min-width: 0;
}

.topbar-stats {
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.app-mark {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  flex: 0 0 auto;
  color: #ffffff;
  border-radius: var(--radius);
  background: var(--accent);
  box-shadow: 0 1px 2px rgba(18, 24, 38, 0.14);
  font-size: 18px;
  font-weight: 700;
}

.stat-pill {
  min-height: 34px;
  gap: 6px;
  padding: 6px 10px;
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: rgba(255, 255, 255, 0.72);
  box-shadow: var(--shadow-sm);
  font-size: 12px;
  line-height: 1;
}

.stat-pill strong {
  max-width: 160px;
  overflow: hidden;
  font-size: 13px;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stat-label {
  color: var(--text-muted);
}

.active-route-pill {
  border-color: var(--accent-border);
  background: var(--accent-soft);
}

.refresh-button {
  min-width: 38px;
  min-height: 38px;
}

.workspace {
  display: grid;
  grid-template-columns: minmax(280px, 380px) minmax(0, 1fr);
  min-height: calc(100vh - 84px);
}

.editor {
  display: grid;
  align-content: start;
  gap: 18px;
  padding: 24px;
}

.spinning {
  animation: spin 0.8s linear infinite;
}

@media (max-width: 900px) {
  .workspace {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .topbar {
    align-items: flex-start;
    flex-direction: column;
    padding: 18px;
  }

  .topbar-actions,
  .topbar-stats {
    justify-content: flex-start;
    width: 100%;
  }

  .editor {
    padding: 18px;
  }
}
</style>
