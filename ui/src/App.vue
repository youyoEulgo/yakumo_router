<script setup lang="ts">
import AppTopbar from './components/AppTopbar.vue';
import MissingConfigPanel from './components/MissingConfigPanel.vue';
import ProviderEditor from './components/ProviderEditor.vue';
import RouteTableEditor from './components/RouteTableEditor.vue';
import SidebarNav from './components/SidebarNav.vue';
import StatusBar from './components/StatusBar.vue';
import { useRouterConfigEditor } from './composables/useRouterConfigEditor';
import { useI18n } from './i18n';

const { locale, localeOptions, setLocale, t } = useI18n();

const {
  activateRouteTable,
  activatingRouteTable,
  activePane,
  activeProtocol,
  configExists,
  createConfig,
  creatingConfig,
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
    <AppTopbar
      :active-route-table="routeTableState.active"
      :current-locale="locale"
      :language-label="t('language')"
      :locale-options="localeOptions"
      :loading="loading"
      :messages="{
        active: t('active'),
        configurationSummary: t('configurationSummary'),
        none: t('none'),
        providers: t('providers'),
        refresh: t('refresh'),
        refreshing: t('refreshing'),
        rules: t('rules'),
        tables: t('tables'),
      }"
      :total-providers="totalProviders"
      :total-routes="totalRoutes"
      :total-route-tables="totalRouteTables"
      :topbar-context="topbarContext"
      @change-locale="setLocale"
      @refresh="loadAll"
    />

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
        <MissingConfigPanel
          v-if="!configExists"
          :creating="creatingConfig"
          :messages="{
            action: t('createConfig'),
            creating: t('creatingConfig'),
            detail: t('missingConfigDetail'),
            title: t('missingConfigTitle'),
          }"
          @create="createConfig"
        />

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

@media (max-width: 900px) {
  .workspace {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .editor {
    padding: 18px;
  }
}
</style>
