<script setup lang="ts">
import { useI18n } from '../i18n';
import type { RouteRule } from '../types';

const props = defineProps<{
  disabled: boolean;
  draggable: boolean;
  dragging: boolean;
  dropPlacement: 'before' | 'after' | null;
  enabled: boolean;
  route: RouteRule;
}>();

const emit = defineEmits<{
  dragend: [];
  dragleave: [];
  dragover: [event: DragEvent];
  dragstart: [event: DragEvent];
  drop: [event: DragEvent];
  move: [direction: -1 | 1];
  remove: [];
  toggle: [enabled: boolean];
}>();

const { t } = useI18n();

function onRouteToggle(event: Event): void {
  emit('toggle', (event.target as HTMLInputElement).checked);
}
</script>

<template>
  <div
    class="card-row route-toggle-row"
    :class="{
      draggable: props.draggable,
      dragging,
      'drop-before': dropPlacement === 'before',
      'drop-after': dropPlacement === 'after',
    }"
    :draggable="props.draggable"
    @dragstart="emit('dragstart', $event)"
    @dragover="emit('dragover', $event)"
    @dragleave="emit('dragleave')"
    @drop="emit('drop', $event)"
    @dragend="emit('dragend')"
  >
    <span class="drag-handle" :class="{ disabled: !enabled || disabled }" aria-hidden="true">
      <span></span>
      <span></span>
      <span></span>
      <span></span>
      <span></span>
      <span></span>
    </span>
    <label class="switch-row">
      <input type="checkbox" :disabled="disabled" :checked="enabled" @change="onRouteToggle" />
      <span class="switch-track" aria-hidden="true"></span>
      <span class="route-toggle-text">
        <strong>{{ route.id }}</strong>
        <small>
          {{ route.match_type ?? 'contains' }} {{ route.match }} / {{ route.provider }}
        </small>
      </span>
    </label>

    <div class="order-actions">
      <button
        type="button"
        class="ghost-button compact"
        :disabled="disabled"
        :title="t('moveUp')"
        :aria-label="t('moveUp')"
        @click="emit('move', -1)"
      >
        ↑
      </button>
      <button
        type="button"
        class="ghost-button compact"
        :disabled="disabled"
        :title="t('moveDown')"
        :aria-label="t('moveDown')"
        @click="emit('move', 1)"
      >
        ↓
      </button>
      <button
        type="button"
        class="ghost-button compact remove-button"
        :disabled="disabled"
        :title="t('removeFromTable')"
        :aria-label="t('removeFromTable')"
        @click="emit('remove')"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<style scoped>
.route-toggle-row {
  grid-template-columns: 18px minmax(0, 1fr) auto;
  align-items: center;
  min-height: 60px;
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
  color: var(--text-soft);
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
  background: var(--switch-track);
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

.remove-button {
  color: var(--danger);
}

.remove-button:hover:not(:disabled) {
  border-color: var(--danger-border);
  background: var(--danger-bg);
}
</style>
