import { onBeforeUnmount, ref } from 'vue';

export function useToast() {
  const statusMessage = ref('');
  const errorMessage = ref('');
  const toastKey = ref(0);
  let toastTimer: ReturnType<typeof window.setTimeout> | undefined;

  function setStatus(message: string): void {
    statusMessage.value = message;
    errorMessage.value = '';
    toastKey.value += 1;
    scheduleToastDismiss();
  }

  function setError(message: string): void {
    errorMessage.value = message;
    statusMessage.value = '';
    toastKey.value += 1;
    scheduleToastDismiss();
  }

  function clearToast(): void {
    statusMessage.value = '';
    errorMessage.value = '';
    if (toastTimer) {
      window.clearTimeout(toastTimer);
      toastTimer = undefined;
    }
  }

  function scheduleToastDismiss(): void {
    if (toastTimer) {
      window.clearTimeout(toastTimer);
    }
    toastTimer = window.setTimeout(() => {
      statusMessage.value = '';
      errorMessage.value = '';
      toastTimer = undefined;
    }, 3200);
  }

  onBeforeUnmount(() => {
    if (toastTimer) {
      window.clearTimeout(toastTimer);
    }
  });

  return {
    clearToast,
    errorMessage,
    setError,
    setStatus,
    statusMessage,
    toastKey,
  };
}
