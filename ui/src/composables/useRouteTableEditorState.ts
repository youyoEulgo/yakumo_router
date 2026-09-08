import { computed, reactive, ref } from 'vue';
import {
  activateRouteTable as activateRouteTableApi,
  deleteRouteTable as deleteRouteTableApi,
  mutateRouteTable as mutateRouteTableApi,
  saveRouteTable as saveRouteTableApi,
} from '../api';
import type { RouteTableMutation } from '../api';
import type { EditorPane, Protocol, RouteTable, RouteTableState } from '../types';

type RouteTableEditorStateOptions = {
  activePane: { value: EditorPane };
  routeTableState: RouteTableState;
  reload: () => Promise<void>;
  clearToast: () => void;
  onError: (message: string) => void;
  onStatus: (message: string) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
};

export function useRouteTableEditorState({
  activePane,
  clearToast,
  onError,
  onStatus,
  reload,
  routeTableState,
  t,
}: RouteTableEditorStateOptions) {
  const selectedRouteTable = ref('');
  const routeTableName = ref('');
  const savingRouteTable = ref(false);
  const activatingRouteTable = ref(false);
  const deletingRouteTable = ref(false);
  const draftRouteTable = reactive<RouteTable>({
    openai: [],
    anthropic: [],
  });

  const selectedTable = computed(() => {
    if (selectedRouteTable.value) {
      return routeTableState.tables[selectedRouteTable.value];
    }

    return activePane.value === 'route-table' ? draftRouteTable : undefined;
  });

  function applyRouteTable(name: string): void {
    activePane.value = 'route-table';
    selectedRouteTable.value = name;
    routeTableName.value = name;
  }

  function resetRouteTableForm(): void {
    activePane.value = 'route-table';
    selectedRouteTable.value = '';
    routeTableName.value = '';
    draftRouteTable.openai = [];
    draftRouteTable.anthropic = [];
    clearToast();
  }

  function clearRouteTableSelection(): void {
    selectedRouteTable.value = '';
    routeTableName.value = '';
    draftRouteTable.openai = [];
    draftRouteTable.anthropic = [];
  }

  function reconcileRouteTableSelection(): void {
    if (selectedRouteTable.value && !routeTableState.tables[selectedRouteTable.value]) {
      clearRouteTableSelection();
    }
    if (!selectedRouteTable.value && routeTableState.active) {
      applyRouteTable(routeTableState.active);
    }
  }

  async function applyRouteTableMutation(mutation: RouteTableMutation): Promise<void> {
    const name = selectedRouteTable.value;
    if (!name) {
      return;
    }

    savingRouteTable.value = true;

    try {
      const result = await mutateRouteTableApi(name, mutation);
      routeTableState.tables[result.name] = result.table;
      onStatus(t('routeTableSaved'));
    } catch (error) {
      onError(error instanceof Error ? error.message : t('failedSaveRouteTable'));
      await reload();
    } finally {
      savingRouteTable.value = false;
    }
  }

  async function addRoutesToTable(protocol: Protocol, ids: string[]): Promise<void> {
    if (ids.length === 0) {
      return;
    }

    await applyRouteTableMutation({ protocol, action: 'add', ids });
  }

  async function removeRouteFromTable(protocol: Protocol, routeId: string): Promise<void> {
    await applyRouteTableMutation({ protocol, action: 'remove', ids: [routeId] });
  }

  async function toggleRouteInTable(
    protocol: Protocol,
    routeId: string,
    enabled: boolean,
  ): Promise<void> {
    const table = selectedTable.value;
    if (!table) {
      return;
    }

    const entry = table[protocol].find((item) => item.id === routeId);
    if (!entry || entry.enabled === enabled) {
      return;
    }

    entry.enabled = enabled;
    await applyRouteTableMutation({ protocol, action: 'update', entries: table[protocol] });
  }

  async function moveRouteInTable(
    protocol: Protocol,
    routeId: string,
    direction: -1 | 1,
  ): Promise<void> {
    const table = selectedTable.value;
    if (!table) {
      return;
    }

    const entries = table[protocol];
    const index = entries.findIndex((entry) => entry.id === routeId);
    const nextIndex = index + direction;
    if (index === -1 || nextIndex < 0 || nextIndex >= entries.length) {
      return;
    }

    const [moved] = entries.splice(index, 1);
    if (moved === undefined) {
      return;
    }

    entries.splice(nextIndex, 0, moved);
    await applyRouteTableMutation({ protocol, action: 'update', entries });
  }

  async function saveRouteTable(): Promise<void> {
    const name = routeTableName.value.trim();
    if (!name) {
      onError(t('routeTableNameRequired'));
      return;
    }

    savingRouteTable.value = true;

    try {
      await saveRouteTableApi(name, selectedTable.value ?? { openai: [], anthropic: [] });
      selectedRouteTable.value = name;
      await reload();
      applyRouteTable(name);
      onStatus(t('routeTableSaved'));
    } catch (error) {
      onError(error instanceof Error ? error.message : t('failedSaveRouteTable'));
    } finally {
      savingRouteTable.value = false;
    }
  }

  async function deleteSelectedRouteTable(): Promise<void> {
    if (!selectedRouteTable.value) {
      return;
    }

    deletingRouteTable.value = true;

    try {
      const deleted = selectedRouteTable.value;
      await deleteRouteTableApi(deleted);
      await reload();
      clearRouteTableSelection();
      onStatus(t('routeTableDeleted', { name: deleted }));
    } catch (error) {
      onError(error instanceof Error ? error.message : t('failedDeleteRouteTable'));
    } finally {
      deletingRouteTable.value = false;
    }
  }

  async function activateRouteTable(): Promise<void> {
    if (!selectedRouteTable.value) {
      return;
    }

    activatingRouteTable.value = true;

    try {
      await activateRouteTableApi(selectedRouteTable.value);
      routeTableState.active = selectedRouteTable.value;
      onStatus(t('routeTableActivated'));
    } catch (error) {
      onError(error instanceof Error ? error.message : t('failedActivateRouteTable'));
    } finally {
      activatingRouteTable.value = false;
    }
  }

  return {
    activateRouteTable,
    activatingRouteTable,
    addRoutesToTable,
    applyRouteTable,
    deleteSelectedRouteTable,
    deletingRouteTable,
    moveRouteInTable,
    reconcileRouteTableSelection,
    removeRouteFromTable,
    resetRouteTableForm,
    routeTableName,
    saveRouteTable,
    savingRouteTable,
    selectedRouteTable,
    selectedTable,
    toggleRouteInTable,
  };
}
