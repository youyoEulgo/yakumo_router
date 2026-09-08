<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from '../i18n';

const props = defineProps<{
  currentValue: string;
  label: string;
  note: string;
  saving: boolean;
  title: string;
}>();

const emit = defineEmits<{
  cancel: [];
  confirm: [value: string];
}>();

const { t } = useI18n();
const value = ref(props.currentValue);
const nextValue = computed(() => value.value.trim());
const canSubmit = computed(
  () => nextValue.value.length > 0 && nextValue.value !== props.currentValue && !props.saving,
);

function submit(): void {
  if (canSubmit.value) {
    emit('confirm', nextValue.value);
  }
}
</script>

<template>
  <div class="dialog-backdrop" @click.self="!saving && emit('cancel')">
    <form class="dialog-card" @submit.prevent="submit">
      <h3>{{ title }}</h3>
      <p class="dialog-note">{{ note }}</p>

      <label class="dialog-field">
        <span>{{ label }}</span>
        <input v-model="value" required autocomplete="off" :disabled="saving" />
      </label>

      <div class="actions dialog-actions">
        <button class="primary-button compact" type="submit" :disabled="!canSubmit">
          {{ saving ? t('saving') : t('confirm') }}
        </button>
        <button
          class="ghost-button compact"
          type="button"
          :disabled="saving"
          @click="emit('cancel')"
        >
          {{ t('cancel') }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgba(15, 23, 42, 0.42);
}

.dialog-card {
  display: grid;
  gap: 14px;
  width: min(420px, 100%);
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: var(--shadow-md);
}

.dialog-card h3 {
  margin: 0;
  color: var(--text);
  font-size: 15px;
}

.dialog-note {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.dialog-actions {
  justify-content: flex-end;
}
</style>
