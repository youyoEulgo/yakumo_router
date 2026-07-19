<script setup lang="ts">
import { useRouteDragSort } from '../composables/useRouteDragSort';
import { protocolLabel, useI18n } from '../i18n';
import type { Protocol, RouteRule, RouteTable } from '../types';
import RouteTableForm from './RouteTableForm.vue';
import RouteToggleRow from './RouteToggleRow.vue';

const props = defineProps<{
  activating: boolean;
  activeRouteTable: string | null;
  deleting: boolean;
  routeTable: RouteTable | undefined;
  routeTableName: string;
  routes: Record<Protocol, RouteRule[]>;
  saving: boolean;
  selectedRouteTable: string;
}>();

const emit = defineEmits<{
  'update:routeTableName': [name: string];
  activate: [];
  delete: [];
  save: [];
  toggleRoute: [protocol: Protocol, routeId: string, enabled: boolean];
  moveRoute: [protocol: Protocol, routeId: string, direction: -1 | 1];
}>();

const { t } = useI18n();

function routeEnabled(table: RouteTable | undefined, protocol: Protocol, routeId: string): boolean {
  return table?.[protocol].includes(routeId) ?? false;
}

function orderedRoutes(
  table: RouteTable | undefined,
  allRoutes: Record<Protocol, RouteRule[]>,
  protocol: Protocol,
): RouteRule[] {
  const ids = table?.[protocol] ?? [];
  const enabled = ids
    .map((id) => allRoutes[protocol].find((route) => route.id === id))
    .filter((route): route is RouteRule => Boolean(route));
  const disabled = allRoutes[protocol].filter((route) => !routeEnabled(table, protocol, route.id));

  return [...enabled, ...disabled];
}

function enabledRouteIds(protocol: Protocol): string[] {
  return props.routeTable?.[protocol] ?? [];
}

const {
  clearDragState,
  clearDropTarget,
  dropPlacement,
  isDragging,
  onDragOver,
  onDragStart,
  onDrop,
} = useRouteDragSort({
  canDragRoute: (protocol, routeId) =>
    !props.saving && routeEnabled(props.routeTable, protocol, routeId),
  enabledRouteIds,
  moveRoute: (protocol, routeId, direction) => emit('moveRoute', protocol, routeId, direction),
});
</script>

<template>
  <section class="panel">
    <div class="panel-header">
      <div>
        <h2>{{ t('routeTable') }}</h2>
        <p v-if="selectedRouteTable" class="panel-note">
          {{ selectedRouteTable }}
          {{ activeRouteTable === selectedRouteTable ? t('isActive') : t('isInactive') }}
        </p>
        <p v-else class="panel-note">{{ t('newRouteTable') }}</p>
      </div>
      <button
        class="ghost-button compact"
        type="button"
        :disabled="!selectedRouteTable || activeRouteTable === selectedRouteTable || activating"
        @click="emit('activate')"
      >
        {{ activating ? t('activating') : t('activate') }}
      </button>
    </div>

    <div v-if="routeTable" class="route-table-layout">
      <RouteTableForm
        :deleting="deleting"
        :route-table-name="routeTableName"
        :saving="saving"
        :selected-route-table="selectedRouteTable"
        @delete="emit('delete')"
        @save="emit('save')"
        @update:route-table-name="emit('update:routeTableName', $event)"
      />

      <div class="route-table-rules">
        <section
          v-for="protocol in ['openai', 'anthropic'] as Protocol[]"
          :key="protocol"
          class="route-table-section"
        >
          <h3>{{ t('rulesSection', { protocol: protocolLabel(protocol) }) }}</h3>
          <div v-if="routes[protocol].length === 0" class="empty-state">
            {{ t('noRules') }}
          </div>
          <div v-else class="route-toggle-list">
            <RouteToggleRow
              v-for="route in orderedRoutes(routeTable, routes, protocol)"
              :key="route.id"
              :disabled="saving"
              :draggable="routeEnabled(routeTable, protocol, route.id) && !saving"
              :dragging="isDragging(protocol, route.id)"
              :drop-placement="dropPlacement(protocol, route.id)"
              :enabled="routeEnabled(routeTable, protocol, route.id)"
              :route="route"
              @dragend="clearDragState"
              @dragleave="clearDropTarget"
              @dragover="onDragOver(protocol, route.id, $event)"
              @dragstart="onDragStart(protocol, route.id, $event)"
              @drop="onDrop(protocol, route.id, $event)"
              @move="emit('moveRoute', protocol, route.id, $event)"
              @toggle="emit('toggleRoute', protocol, route.id, $event)"
            />
          </div>
        </section>
      </div>
    </div>
  </section>
</template>

<style scoped>
.panel {
  display: grid;
  gap: 18px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(249, 251, 252, 0.86)), var(--surface);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.7) inset,
    var(--shadow-sm);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.panel-header h2,
.panel-header p,
.route-table-section h3 {
  margin: 0;
}

.panel-header h2 {
  color: var(--text);
  font-size: 15px;
  line-height: 1.3;
  letter-spacing: 0;
}

.panel-note {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  min-height: 26px;
  margin-top: 8px;
  padding: 4px 8px;
  color: #536276;
  border-left: 3px solid var(--accent-border);
  border-radius: 0 var(--radius) var(--radius) 0;
  background: rgba(237, 244, 255, 0.68);
  font-size: 12px;
  line-height: 1.4;
}

.route-table-layout,
.route-table-rules,
.route-toggle-list,
.route-table-section {
  display: grid;
  align-content: start;
  gap: 10px;
}

.route-table-layout {
  gap: 16px;
}

.route-table-section {
  gap: 12px;
  padding-top: 4px;
}

.route-table-section h3 {
  color: #334155;
  font-size: 13px;
  line-height: 1.3;
}
</style>
