<script setup lang="ts">
import { ref } from 'vue';
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

const draggingRoute = ref<{ protocol: Protocol; routeId: string } | null>(null);
const dropTarget = ref<{ protocol: Protocol; routeId: string; placement: 'before' | 'after' } | null>(null);

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

function onDragStart(protocol: Protocol, routeId: string, event: DragEvent): void {
  if (props.saving || !routeEnabled(props.routeTable, protocol, routeId)) {
    event.preventDefault();
    return;
  }

  draggingRoute.value = { protocol, routeId };
  event.dataTransfer?.setData('text/plain', routeId);
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
  }
}

function onDragOver(protocol: Protocol, routeId: string, event: DragEvent): void {
  if (
    !draggingRoute.value ||
    draggingRoute.value.protocol !== protocol ||
    draggingRoute.value.routeId === routeId ||
    !routeEnabled(props.routeTable, protocol, routeId)
  ) {
    return;
  }

  event.preventDefault();
  const target = event.currentTarget as HTMLElement;
  const { top, height } = target.getBoundingClientRect();
  const placement = event.clientY < top + height / 2 ? 'before' : 'after';
  dropTarget.value = { protocol, routeId, placement };
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
}

function onDrop(protocol: Protocol, targetRouteId: string, event: DragEvent): void {
  event.preventDefault();
  const dragged = draggingRoute.value;
  if (!dragged || dragged.protocol !== protocol || dragged.routeId === targetRouteId) {
    clearDragState();
    return;
  }

  const ids = enabledRouteIds(protocol);
  const fromIndex = ids.indexOf(dragged.routeId);
  const targetIndex = ids.indexOf(targetRouteId);
  if (fromIndex === -1 || targetIndex === -1) {
    clearDragState();
    return;
  }

  const placement = dropTarget.value?.placement ?? 'before';
  const toIndex = placement === 'before' ? targetIndex : targetIndex + 1;
  const adjustedToIndex = fromIndex < toIndex ? toIndex - 1 : toIndex;
  if (fromIndex === adjustedToIndex) {
    clearDragState();
    return;
  }

  const direction: -1 | 1 = fromIndex < adjustedToIndex ? 1 : -1;
  for (let index = fromIndex; index !== adjustedToIndex; index += direction) {
    emit('moveRoute', protocol, dragged.routeId, direction);
  }
  clearDragState();
}

function clearDragState(): void {
  draggingRoute.value = null;
  dropTarget.value = null;
}

function isDragging(protocol: Protocol, routeId: string): boolean {
  return draggingRoute.value?.protocol === protocol && draggingRoute.value.routeId === routeId;
}

function dropPlacement(protocol: Protocol, routeId: string): 'before' | 'after' | null {
  if (dropTarget.value?.protocol !== protocol || dropTarget.value.routeId !== routeId) {
    return null;
  }

  return dropTarget.value.placement;
}
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
      <button class="ghost-button compact" type="button"
        :disabled="!selectedRouteTable || activeRouteTable === selectedRouteTable || activating"
        @click="emit('activate')">
        {{ activating ? 'Activating...' : 'Activate' }}
      </button>
    </div>

    <div v-if="routeTable" class="route-table-layout">
      <form class="route-table-form" @submit.prevent="emit('save')">
        <label>
          <span>Name</span>
          <input :value="routeTableName" required autocomplete="off" placeholder="default"
            @input="emit('update:routeTableName', ($event.target as HTMLInputElement).value)" />
        </label>

        <div class="actions">
          <button class="primary-button" type="submit" :disabled="saving">
            <svg class="button-icon" aria-hidden="true" viewBox="0 0 24 24" fill="none">
              <path
                d="M5 4h12l2 2v14H5V4Z"
                stroke="currentColor"
                stroke-width="2"
                stroke-linejoin="round"
              />
              <path d="M8 4v6h8V4" stroke="currentColor" stroke-width="2" stroke-linejoin="round" />
              <path d="M8 20v-6h8v6" stroke="currentColor" stroke-width="2" stroke-linejoin="round" />
            </svg>
            {{ saving ? 'Saving...' : 'Save' }}
          </button>
          <button class="danger-button" type="button" :disabled="!selectedRouteTable || deleting"
            @click="emit('delete')">
            {{ deleting ? 'Deleting...' : 'Delete' }}
          </button>
        </div>
      </form>

      <div class="route-table-rules">
        <section v-for="protocol in (['openai', 'anthropic'] as Protocol[])" :key="protocol"
          class="route-table-section">
          <h3>{{ protocolLabels[protocol] }} Rules</h3>
          <div v-if="routes[protocol].length === 0" class="empty-state">
            No rules configured.
          </div>
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
              @dragleave="dropTarget = null"
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
                <input type="checkbox" :disabled="saving" :checked="routeEnabled(routeTable, protocol, route.id)"
                  @change="onRouteToggle(protocol, route.id, $event)" />
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
                <button type="button" class="ghost-button compact"
                  :disabled="saving || !routeEnabled(routeTable, protocol, route.id)" title="Move up"
                  aria-label="Move up" @click="emit('moveRoute', protocol, route.id, -1)">
                  ↑
                </button>
                <button type="button" class="ghost-button compact"
                  :disabled="saving || !routeEnabled(routeTable, protocol, route.id)" title="Move down"
                  aria-label="Move down" @click="emit('moveRoute', protocol, route.id, 1)">
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
