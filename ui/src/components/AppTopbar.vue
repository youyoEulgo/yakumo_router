<script setup lang="ts">
defineProps<{
  activeRouteTable: string | null;
  loading: boolean;
  totalProviders: number;
  totalRoutes: number;
  totalRouteTables: number;
  topbarContext: string;
}>();

const emit = defineEmits<{
  refresh: [];
}>();
</script>

<template>
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
          <strong>{{ activeRouteTable ?? 'None' }}</strong>
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
        @click="emit('refresh')"
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
</template>

<style scoped>
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

h1,
p {
  margin: 0;
}

h1 {
  font-size: 22px;
  line-height: 1.2;
  letter-spacing: 0;
}

p {
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 13px;
}

.spinning {
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
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
}
</style>
