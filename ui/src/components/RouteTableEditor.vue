<script setup lang="ts">
import { useRouteDragSort } from '../composables/useRouteDragSort';
import SaveIcon from './SaveIcon.vue';
import type { Protocol, RouteRule, RouteTable } from '../types';
import { protocolLabels } from '../types';

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

function onRouteToggle(protocol: Protocol, routeId: string, event: Event): void {
  emit('toggleRoute', protocol, routeId, (event.target as HTMLInputElement).checked);
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
        <h2>Route Table</h2>
        <p v-if="selectedRouteTable">
          {{ selectedRouteTable }}
          {{ activeRouteTable === selectedRouteTable ? 'is active' : 'is inactive' }}
        </p>
        <p v-else>New route table</p>
      </div>
      <button
        class="ghost-button compact"
        type="button"
        :disabled="!selectedRouteTable || activeRouteTable === selectedRouteTable || activating"
        @click="emit('activate')"
      >
        {{ activating ? 'Activating...' : 'Activate' }}
      </button>
    </div>

    <div v-if="routeTable" class="route-table-layout">
      <form class="route-table-form" @submit.prevent="emit('save')">
        <label>
          <span>Name</span>
          <input
            :value="routeTableName"
            required
            autocomplete="off"
            placeholder="default"
            @input="emit('update:routeTableName', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <div class="actions">
          <button class="primary-button" type="submit" :disabled="saving">
            <SaveIcon />
            {{ saving ? 'Saving...' : 'Save' }}
          </button>
          <button
            class="danger-button"
            type="button"
            :disabled="!selectedRouteTable || deleting"
            @click="emit('delete')"
          >
            {{ deleting ? 'Deleting...' : 'Delete' }}
          </button>
        </div>
      </form>

      <div class="route-table-rules">
        <section
          v-for="protocol in ['openai', 'anthropic'] as Protocol[]"
          :key="protocol"
          class="route-table-section"
        >
          <h3>{{ protocolLabels[protocol] }} Rules</h3>
          <div v-if="routes[protocol].length === 0" class="empty-state">No rules configured.</div>
          <div v-else class="route-toggle-list">
            <div
              v-for="route in orderedRoutes(routeTable, routes, protocol)"
              :key="route.id"
              class="route-toggle-row"
              :class="{
                draggable: routeEnabled(routeTable, protocol, route.id) && !saving,
                dragging: isDragging(protocol, route.id),
                'drop-before': dropPlacement(protocol, route.id) === 'before',
                'drop-after': dropPlacement(protocol, route.id) === 'after',
              }"
              :draggable="routeEnabled(routeTable, protocol, route.id) && !saving"
              @dragstart="onDragStart(protocol, route.id, $event)"
              @dragover="onDragOver(protocol, route.id, $event)"
              @dragleave="clearDropTarget"
              @drop="onDrop(protocol, route.id, $event)"
              @dragend="clearDragState"
            >
              <span
                class="drag-handle"
                :class="{ disabled: !routeEnabled(routeTable, protocol, route.id) || saving }"
                aria-hidden="true"
              >
                <span></span>
                <span></span>
                <span></span>
                <span></span>
                <span></span>
                <span></span>
              </span>
              <label class="switch-row">
                <input
                  type="checkbox"
                  :disabled="saving"
                  :checked="routeEnabled(routeTable, protocol, route.id)"
                  @change="onRouteToggle(protocol, route.id, $event)"
                />
                <span class="switch-track" aria-hidden="true"></span>
                <span class="route-toggle-text">
                  <strong>{{ route.id }}</strong>
                  <small>
                    {{ route.match_type ?? 'contains' }} {{ route.match }} /
                    {{ route.provider }}
                  </small>
                </span>
              </label>

              <div class="order-actions">
                <button
                  type="button"
                  class="ghost-button compact"
                  :disabled="saving || !routeEnabled(routeTable, protocol, route.id)"
                  title="Move up"
                  aria-label="Move up"
                  @click="emit('moveRoute', protocol, route.id, -1)"
                >
                  ↑
                </button>
                <button
                  type="button"
                  class="ghost-button compact"
                  :disabled="saving || !routeEnabled(routeTable, protocol, route.id)"
                  title="Move down"
                  aria-label="Move down"
                  @click="emit('moveRoute', protocol, route.id, 1)"
                >
                  ↓
                </button>
              </div>
            </div>
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

.panel-header p {
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 13px;
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

.route-table-form {
  display: grid;
  max-width: 720px;
  gap: 16px;
}

.route-table-form label {
  display: grid;
  gap: 7px;
  color: #425066;
  font-size: 12px;
  font-weight: 700;
}

.route-table-form input {
  width: 100%;
  min-height: 42px;
  padding: 0 12px;
  color: var(--text);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: inset 0 1px 0 rgba(18, 24, 38, 0.03);
  transition:
    border-color 0.16s ease,
    box-shadow 0.16s ease,
    background-color 0.16s ease;
}

.route-table-form input:focus {
  border-color: var(--accent);
  outline: 3px solid rgba(39, 100, 216, 0.16);
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

.route-toggle-row {
  position: relative;
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  gap: 6px;
  align-items: center;
  width: 100%;
  min-height: 60px;
  padding: 12px 13px;
  text-align: left;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(250, 252, 253, 0.94)),
    var(--surface-raised);
  box-shadow: var(--shadow-sm);
  transition:
    border-color 0.16s ease,
    background-color 0.16s ease,
    box-shadow 0.16s ease,
    transform 0.16s ease;
}

.route-toggle-row:hover {
  border-color: var(--accent-border);
  background: var(--surface);
  box-shadow: var(--shadow-md);
  transform: translateY(-1px);
}

.route-toggle-row.draggable {
  cursor: grab;
}

.route-toggle-row.dragging {
  opacity: 0.48;
}

.route-toggle-row.draggable:active {
  cursor: grabbing;
}

.route-toggle-row.drop-before::before,
.route-toggle-row.drop-after::before {
  position: absolute;
  right: 12px;
  left: 12px;
  z-index: 1;
  height: 3px;
  border-radius: 999px;
  background: var(--accent);
  box-shadow: 0 0 0 3px rgba(39, 100, 216, 0.12);
  content: '';
}

.route-toggle-row.drop-before::before {
  top: -7px;
}

.route-toggle-row.drop-after::before {
  bottom: -7px;
}

.drag-handle {
  display: grid;
  grid-template-columns: repeat(2, 3px);
  gap: 3px;
  justify-content: center;
  align-content: center;
  width: 18px;
  min-height: 34px;
  color: #8a95a7;
}

.drag-handle span {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: currentColor;
}

.route-toggle-row.draggable:hover .drag-handle {
  color: var(--accent);
}

.drag-handle.disabled {
  opacity: 0.35;
}

.switch-row {
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
}

.switch-row input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.switch-track {
  position: relative;
  width: 42px;
  height: 24px;
  border-radius: 999px;
  background: #c7d0dc;
  box-shadow: inset 0 1px 2px rgba(18, 24, 38, 0.12);
  transition: background-color 0.16s ease;
}

.switch-track::after {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #ffffff;
  box-shadow: 0 1px 3px rgba(18, 24, 38, 0.28);
  content: '';
  transition: transform 0.16s ease;
}

.switch-row input:checked + .switch-track {
  background: var(--accent);
}

.switch-row input:checked + .switch-track::after {
  transform: translateX(18px);
}

.route-toggle-text {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.route-toggle-text strong,
.route-toggle-text small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.route-toggle-text strong {
  color: var(--text);
  font-size: 14px;
}

.route-toggle-text small {
  color: var(--text-muted);
  font-size: 12px;
}

.order-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

@media (max-width: 900px) {
  .route-table-form {
    grid-template-columns: 1fr;
  }
}
</style>
