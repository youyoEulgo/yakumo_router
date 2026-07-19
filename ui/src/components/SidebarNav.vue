<script setup lang="ts">
import { reactive } from 'vue';
import type {
  EditorPane,
  ProviderConfig,
  ProviderTables,
  Protocol,
  RouteTables,
  RouteTableState,
} from '../types';
import { protocolLabels } from '../types';

const props = defineProps<{
  activePane: EditorPane;
  activeProtocol: Protocol;
  loading: boolean;
  providers: ProviderTables;
  routes: RouteTables;
  routeTables: RouteTableState;
  selectedProvider: string;
  selectedRouteTable: string;
}>();

const emit = defineEmits<{
  newProvider: [protocol: Protocol];
  selectProvider: [protocol: Protocol, name: string, provider: ProviderConfig];
  newRouteTable: [];
  selectRouteTable: [name: string];
}>();

const collapsed = reactive<Record<Protocol | 'routeTables', boolean>>({
  openai: false,
  anthropic: false,
  routeTables: false,
});

function providerEntries(protocol: Protocol): [string, ProviderConfig][] {
  return Object.entries(props.providers[protocol]).sort(([left], [right]) =>
    left.localeCompare(right),
  );
}

function providerRouteCount(protocol: Protocol, provider: string): number {
  return props.routes[protocol].filter((route) => route.provider === provider).length;
}

function routeTableEntries(): string[] {
  return Object.keys(props.routeTables.tables).sort((left, right) => left.localeCompare(right));
}

function routeTableRuleCount(name: string, protocol: Protocol): number {
  return props.routeTables.tables[name]?.[protocol].length ?? 0;
}
</script>

<template>
  <aside class="sidebar">
    <template v-for="protocol in ['openai', 'anthropic'] as Protocol[]" :key="protocol">
      <button
        type="button"
        class="list-header collapsible-header"
        :aria-expanded="!collapsed[protocol]"
        @click="collapsed[protocol] = !collapsed[protocol]"
      >
        <h2>{{ protocolLabels[protocol] }} Providers</h2>
        <span
          class="collapse-button"
          :class="{ collapsed: collapsed[protocol] }"
          aria-hidden="true"
        >
          ▾
        </span>
      </button>

      <template v-if="!collapsed[protocol]">
        <div v-if="loading" class="empty-state">Loading providers...</div>
        <div v-else-if="providerEntries(protocol).length === 0" class="empty-state">
          No providers configured.
        </div>
        <template v-else>
          <button
            v-for="[name, provider] in providerEntries(protocol)"
            :key="`${protocol}-${name}`"
            type="button"
            class="provider-row"
            :class="{
              selected:
                activePane === 'provider' &&
                activeProtocol === protocol &&
                selectedProvider === name,
            }"
            @click="emit('selectProvider', protocol, name, provider)"
          >
            <span class="provider-name">{{ name }}</span>
            <span class="provider-url">{{ provider.base_url }}</span>
            <span class="provider-count">{{ providerRouteCount(protocol, name) }} rules</span>
          </button>
        </template>

        <button
          type="button"
          class="provider-row new-row"
          :aria-label="`New ${protocolLabels[protocol]} provider`"
          @click="emit('newProvider', protocol)"
        >
          <span class="new-row-plus" aria-hidden="true">+</span>
        </button>
      </template>
    </template>

    <button
      type="button"
      class="list-header collapsible-header"
      :aria-expanded="!collapsed.routeTables"
      @click="collapsed.routeTables = !collapsed.routeTables"
    >
      <h2>Route Tables</h2>
      <span
        class="collapse-button"
        :class="{ collapsed: collapsed.routeTables }"
        aria-hidden="true"
      >
        ▾
      </span>
    </button>

    <template v-if="!collapsed.routeTables">
      <div v-if="routeTableEntries().length === 0" class="empty-state">
        No route tables configured.
      </div>
      <template v-else>
        <button
          v-for="name in routeTableEntries()"
          :key="name"
          type="button"
          class="provider-row"
          :class="{
            selected: activePane === 'route-table' && selectedRouteTable === name,
            active: routeTables.active === name,
          }"
          @click="emit('selectRouteTable', name)"
        >
          <span class="provider-name">{{ name }}</span>
          <span class="provider-url">
            {{ routeTables.active === name ? 'Active route table' : 'Inactive' }}
          </span>
          <span class="provider-row-meta">
            <span v-if="routeTables.active === name" class="active-badge">Active</span>
            <span class="provider-count">
              {{ routeTableRuleCount(name, 'openai') }} OpenAI /
              {{ routeTableRuleCount(name, 'anthropic') }} Anthropic rules
            </span>
          </span>
        </button>
      </template>

      <button
        type="button"
        class="provider-row new-row"
        aria-label="New route table"
        @click="emit('newRouteTable')"
      >
        <span class="new-row-plus" aria-hidden="true">+</span>
      </button>
    </template>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 20px;
  border-right: 1px solid var(--border);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.72), rgba(243, 247, 251, 0.84)), var(--surface);
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  min-height: 40px;
  margin-top: 6px;
  padding: 7px 8px;
  color: inherit;
  text-align: left;
  border: 0;
  border-radius: var(--radius);
  background: transparent;
  transition:
    background-color 0.16s ease,
    color 0.16s ease;
}

.list-header:first-child {
  margin-top: 0;
}

.list-header:hover:not(:disabled) {
  color: var(--accent-strong);
  background: rgba(237, 244, 255, 0.92);
}

.list-header:focus-visible {
  outline: 3px solid rgba(39, 100, 216, 0.16);
}

.list-header h2 {
  margin: 0;
  color: var(--text);
  font-size: 15px;
  line-height: 1.3;
  letter-spacing: 0;
}

.provider-row {
  position: relative;
  display: grid;
  gap: 6px;
  width: 100%;
  min-height: 72px;
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

.provider-row:hover {
  border-color: var(--accent-border);
  background: var(--surface);
  box-shadow: var(--shadow-md);
  transform: translateY(-1px);
}

.provider-row.selected {
  border-color: var(--accent);
  background: var(--accent-soft);
  box-shadow:
    inset 4px 0 0 var(--accent),
    var(--shadow-sm);
}

.provider-row.active {
  border-color: var(--accent-border);
  background:
    linear-gradient(180deg, rgba(237, 244, 255, 0.98), rgba(247, 250, 255, 0.96)),
    var(--accent-soft);
  box-shadow:
    inset 4px 0 0 var(--accent),
    0 0 0 1px rgba(39, 100, 216, 0.08),
    var(--shadow-sm);
}

.provider-row.active.selected {
  border-color: var(--accent);
  box-shadow:
    inset 4px 0 0 var(--accent),
    0 0 0 1px rgba(39, 100, 216, 0.14),
    var(--shadow-md);
}

.provider-name {
  min-width: 0;
  overflow: hidden;
  color: var(--text);
  font-size: 14px;
  font-weight: 700;
  line-height: 1.3;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.provider-url {
  overflow: hidden;
  color: var(--text-muted);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.provider-count {
  justify-self: start;
  min-height: 22px;
  padding: 3px 8px;
  color: #4b5d73;
  border: 1px solid #e2e8f0;
  border-radius: 999px;
  background: var(--surface-subtle);
  font-size: 12px;
  line-height: 1.2;
}

.provider-row-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.active-badge {
  min-height: 22px;
  padding: 3px 8px;
  color: #ffffff;
  border: 1px solid var(--accent);
  border-radius: 999px;
  background: var(--accent);
  font-size: 12px;
  font-weight: 600;
  line-height: 1.2;
}

.new-row {
  place-items: center;
  min-height: 56px;
  color: var(--text-muted);
  border-style: dashed;
  background: rgba(255, 255, 255, 0.6);
  box-shadow: none;
}

.new-row:hover {
  color: var(--accent);
  border-color: var(--accent-border);
  background: var(--accent-soft);
}

.new-row-plus {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: 1px solid currentColor;
  border-radius: 50%;
  font-size: 22px;
  font-weight: 600;
  line-height: 1;
}

@media (max-width: 900px) {
  .sidebar {
    border-right: 0;
    border-bottom: 1px solid var(--border);
  }
}

@media (max-width: 760px) {
  .sidebar {
    padding: 18px;
  }
}
</style>
