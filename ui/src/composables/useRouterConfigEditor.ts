import { computed, onMounted, ref } from 'vue';
import type { EditorPane } from '../types';
import { protocolLabels } from '../types';
import { useConfigState } from './useConfigState';
import { useProviderRuleEditor } from './useProviderRuleEditor';
import { useRouteTableEditorState } from './useRouteTableEditorState';
import { useToast } from './useToast';

export function useRouterConfigEditor() {
  const activePane = ref<EditorPane>('provider');
  const { clearToast, errorMessage, setError, setStatus, statusMessage, toastKey } = useToast();

  const config = useConfigState({
    onError: setError,
  });

  const providerEditor = useProviderRuleEditor({
    activePane,
    clearToast,
    onError: setError,
    onStatus: setStatus,
    reload: loadAll,
    routes: config.routes,
  });

  const routeTableEditor = useRouteTableEditorState({
    activePane,
    clearToast,
    onError: setError,
    onStatus: setStatus,
    reload: loadAll,
    routeTableState: config.routeTableState,
  });

  const topbarContext = computed(() => {
    if (activePane.value === 'route-table') {
      return routeTableEditor.selectedRouteTable.value
        ? `Route table / ${routeTableEditor.selectedRouteTable.value}`
        : 'Route table / new';
    }

    const provider = providerEditor.selectedProvider.value || 'new provider';
    return `${protocolLabels[providerEditor.activeProtocol.value]} / ${provider}`;
  });

  async function loadAll(): Promise<void> {
    errorMessage.value = '';
    await config.loadAll();
    routeTableEditor.reconcileRouteTableSelection();
  }

  onMounted(() => {
    void loadAll();
  });

  return {
    ...config,
    ...providerEditor,
    ...routeTableEditor,
    activePane,
    errorMessage,
    loadAll,
    statusMessage,
    toastKey,
    topbarContext,
  };
}
