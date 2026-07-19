import { computed, onMounted, ref } from 'vue';
import type { EditorPane } from '../types';
import { protocolLabel, useI18n } from '../i18n';
import { useConfigState } from './useConfigState';
import { useProviderRuleEditor } from './useProviderRuleEditor';
import { useRouteTableEditorState } from './useRouteTableEditorState';
import { useToast } from './useToast';

export function useRouterConfigEditor() {
  const activePane = ref<EditorPane>('provider');
  const { clearToast, errorMessage, setError, setStatus, statusMessage, toastKey } = useToast();
  const { t } = useI18n();

  const config = useConfigState({
    onError: setError,
    t,
  });

  const providerEditor = useProviderRuleEditor({
    activePane,
    clearToast,
    onError: setError,
    onStatus: setStatus,
    reload: loadAll,
    routes: config.routes,
    t,
  });

  const routeTableEditor = useRouteTableEditorState({
    activePane,
    clearToast,
    onError: setError,
    onStatus: setStatus,
    reload: loadAll,
    routeTableState: config.routeTableState,
    t,
  });

  const topbarContext = computed(() => {
    if (activePane.value === 'route-table') {
      return routeTableEditor.selectedRouteTable.value
        ? t('routeTableContext', { name: routeTableEditor.selectedRouteTable.value })
        : t('routeTableNewContext');
    }

    const provider = providerEditor.selectedProvider.value || t('newProvider');
    return `${protocolLabel(providerEditor.activeProtocol.value)} / ${provider}`;
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
