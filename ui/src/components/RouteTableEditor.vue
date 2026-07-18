<script setup lang="ts">
import type { Protocol, RouteRule, RouteTable } from '../types';
import { protocolLabels } from '../types';

defineProps<{
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
            {{ saving ? 'Saving...' : 'Save Route Table' }}
          </button>
          <button
            class="danger-button"
            type="button"
            :disabled="!selectedRouteTable || deleting"
            @click="emit('delete')"
          >
            {{ deleting ? 'Deleting...' : 'Delete Route Table' }}
          </button>
        </div>
      </form>

      <div class="route-table-rules">
        <section
          v-for="protocol in (['openai', 'anthropic'] as Protocol[])"
          :key="protocol"
          class="route-table-section"
        >
          <h3>{{ protocolLabels[protocol] }} Rules</h3>
          <div v-if="routes[protocol].length === 0" class="empty-state">
            No rules configured.
          </div>
          <div v-else class="route-toggle-list">
            <div
              v-for="route in orderedRoutes(routeTable, routes, protocol)"
              :key="route.id"
              class="route-toggle-row"
            >
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
