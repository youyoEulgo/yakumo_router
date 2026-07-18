<script setup lang="ts">
import type { Protocol, RouteRule } from '../types';
import { protocolLabels } from '../types';

defineProps<{
  activeProtocol: Protocol;
  deletingProvider: boolean;
  isEditingProvider: boolean;
  isEditingRoute: boolean;
  providerForm: {
    name: string;
    base_url: string;
    api_key: string;
  };
  providerRoutes: RouteRule[];
  routeForm: RouteRule;
  savingProvider: boolean;
  savingRoute: boolean;
  selectedProvider: string;
  selectedRouteId: string;
}>();

const showApiKey = defineModel<boolean>('showApiKey', { required: true });

const emit = defineEmits<{
  updateProviderField: [field: 'name' | 'base_url' | 'api_key', value: string];
  updateRouteField: [field: keyof RouteRule, value: boolean | string];
  saveProvider: [];
  deleteProvider: [];
  resetRoute: [];
  selectRoute: [route: RouteRule];
  saveRoute: [];
  deleteRoute: [];
}>();

function updateProviderField(field: 'name' | 'base_url' | 'api_key', event: Event): void {
  emit('updateProviderField', field, (event.target as HTMLInputElement).value.trim());
}

function updateRouteTextField(field: keyof RouteRule, event: Event): void {
  emit('updateRouteField', field, (event.target as HTMLInputElement | HTMLSelectElement).value.trim());
}

function updateForwardOnly(event: Event): void {
  emit('updateRouteField', 'forward_only', (event.target as HTMLInputElement).checked);
}
</script>

<template>
  <section class="panel">
    <div class="panel-header">
      <div>
        <h2>{{ isEditingProvider ? 'Edit Provider' : 'New Provider' }}</h2>
        <p>{{ protocolLabels[activeProtocol] }} provider settings</p>
      </div>
    </div>

    <form class="form-grid" @submit.prevent="emit('saveProvider')">
      <label>
        <span>Name</span>
        <input
          :value="providerForm.name"
          required
          autocomplete="off"
          placeholder="openrouter"
          @input="updateProviderField('name', $event)"
        />
      </label>

      <label>
        <span>Base URL</span>
        <input
          :value="providerForm.base_url"
          required
          autocomplete="off"
          placeholder="https://openrouter.ai/api/v1"
          @input="updateProviderField('base_url', $event)"
        />
      </label>

      <label>
        <span>API Key</span>
        <span class="secret-field">
          <input
            :value="providerForm.api_key"
            required
            autocomplete="off"
            placeholder="sk-..."
            :type="showApiKey ? 'text' : 'password'"
            @input="updateProviderField('api_key', $event)"
          />
          <button
            class="icon-button"
            type="button"
            :aria-label="showApiKey ? 'Hide API key' : 'Show API key'"
            :title="showApiKey ? 'Hide API key' : 'Show API key'"
            @click="showApiKey = !showApiKey"
          >
            <svg
              v-if="showApiKey"
              aria-hidden="true"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M17.94 17.94A10.94 10.94 0 0 1 12 20C7 20 2.73 16.89 1 12a11.74 11.74 0 0 1 5.06-5.94" />
              <path d="M10.59 10.59a2 2 0 0 0 2.82 2.82" />
              <path d="m3 3 18 18" />
              <path d="M14.12 5.14A10.93 10.93 0 0 1 23 12a11.76 11.76 0 0 1-2.27 3.46" />
            </svg>
            <svg
              v-else
              aria-hidden="true"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7Z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </button>
        </span>
      </label>

      <div class="actions">
        <button class="primary-button" type="submit" :disabled="savingProvider">
          {{ savingProvider ? 'Saving...' : isEditingProvider ? 'Save Provider' : 'Create Provider' }}
        </button>
        <button
          class="danger-button"
          type="button"
          :disabled="!isEditingProvider || deletingProvider"
          @click="emit('deleteProvider')"
        >
          {{ deletingProvider ? 'Deleting...' : 'Delete Provider' }}
        </button>
      </div>
    </form>
  </section>

  <section class="panel" :class="{ muted: !selectedProvider }">
    <div class="panel-header">
      <div>
        <h2>Rules</h2>
        <p v-if="selectedProvider">Routes using {{ selectedProvider }}</p>
        <p v-else>Select a provider to edit rules.</p>
      </div>
      <button
        class="primary-button compact"
        type="button"
        :disabled="!selectedProvider"
        @click="emit('resetRoute')"
      >
        New Rule
      </button>
    </div>

    <div v-if="selectedProvider" class="rules-layout">
      <div class="rule-list">
        <div v-if="providerRoutes.length === 0" class="empty-state">
          No rules for this provider.
        </div>
        <template v-else>
          <button
            v-for="route in providerRoutes"
            :key="route.id"
            type="button"
            class="route-row"
            :class="{ selected: selectedRouteId === route.id }"
            @click="emit('selectRoute', route)"
          >
            <span class="route-id">{{ route.id }}</span>
            <span class="route-detail">
              {{ route.match_type ?? 'contains' }} {{ route.match }}
              {{ route.forward_only ? '-> forward only' : `-> ${route.model}` }}
            </span>
          </button>
        </template>
      </div>

      <form class="form-grid" @submit.prevent="emit('saveRoute')">
        <label>
          <span>ID</span>
          <input
            :value="routeForm.id"
            required
            autocomplete="off"
            placeholder="openai-gpt"
            @input="updateRouteTextField('id', $event)"
          />
        </label>

        <label>
          <span>Match</span>
          <input
            :value="routeForm.match"
            required
            autocomplete="off"
            placeholder="gpt"
            @input="updateRouteTextField('match', $event)"
          />
        </label>

        <label>
          <span>Match Type</span>
          <select :value="routeForm.match_type" @change="updateRouteTextField('match_type', $event)">
            <option value="contains">Contains</option>
            <option value="exact">Exact</option>
            <option value="regex">Regex</option>
          </select>
        </label>

        <label>
          <span>Upstream Model</span>
          <input
            :value="routeForm.model"
            :required="!routeForm.forward_only"
            autocomplete="off"
            :disabled="routeForm.forward_only"
            placeholder="openai/gpt-4.1"
            @input="updateRouteTextField('model', $event)"
          />
        </label>

        <label class="checkbox-row">
          <input
            :checked="routeForm.forward_only"
            type="checkbox"
            @change="updateForwardOnly"
          />
          <span>Forward only</span>
        </label>

        <div class="actions">
          <button class="primary-button" type="submit" :disabled="savingRoute">
            {{ savingRoute ? 'Saving...' : isEditingRoute ? 'Save Rule' : 'Create Rule' }}
          </button>
          <button
            class="danger-button"
            type="button"
            :disabled="!isEditingRoute || savingRoute"
            @click="emit('deleteRoute')"
          >
            Delete Rule
          </button>
        </div>
      </form>
    </div>
  </section>
</template>
