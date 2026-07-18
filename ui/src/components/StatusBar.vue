<script setup lang="ts">
defineProps<{
  error: string;
  status: string;
}>();
</script>

<template>
  <p v-if="status" class="status">{{ status }}</p>
  <p v-if="error" class="error">{{ error }}</p>
</template>
