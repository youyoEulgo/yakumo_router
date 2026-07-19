import { computed, reactive, ref } from 'vue';
import {
  activateRouteTable as activateRouteTableApi,
  deleteRouteTable as deleteRouteTableApi,
  saveRouteTable as saveRouteTableApi,
} from '../api';
import type { EditorPane, Protocol, RouteTable, RouteTableState } from '../types';

type RouteTableEditorStateOptions = {
  activePane: { value: EditorPane };
  routeTableState: RouteTableState;
  reload: () => Promise<void>;
  clearToast: () => void;
  onError: (message: string) => void;
  onStatus: (message: string) => void;
};

export function useRouteTableEditorState({
  activePane,
  clearToast,
  onError,
  onStatus,
  reload,
  routeTableState,
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

  async function persistSelectedRouteTableChange(): Promise<void> {
    if (!selectedRouteTable.value || !selectedTable.value) {
      return;
    }

    savingRouteTable.value = true;

    try {
      await saveRouteTableApi(selectedRouteTable.value, selectedTable.value);
      onStatus('Route table saved.');
    } catch (error) {
      onError(error instanceof Error ? error.message : 'Failed to save route table.');
    } finally {
      savingRouteTable.value = false;
    }
  }

  async function toggleRouteInTable(
    protocol: Protocol,
    routeId: string,
    enabled: boolean,
  ): Promise<void> {
    if (!selectedTable.value) {
      return;
    }

    const ids = selectedTable.value[protocol];
    const index = ids.indexOf(routeId);
    if (enabled && index === -1) {
      ids.push(routeId);
    } else if (!enabled && index !== -1) {
      ids.splice(index, 1);
    } else {
      return;
    }

    await persistSelectedRouteTableChange();
  }

  async function moveRouteInTable(
    protocol: Protocol,
    routeId: string,
    direction: -1 | 1,
  ): Promise<void> {
    if (!selectedTable.value) {
      return;
    }

    const ids = selectedTable.value[protocol];
    const index = ids.indexOf(routeId);
    const nextIndex = index + direction;
    if (index === -1 || nextIndex < 0 || nextIndex >= ids.length) {
      return;
    }

    ids.splice(index, 1);
    ids.splice(nextIndex, 0, routeId);
    await persistSelectedRouteTableChange();
  }

  async function saveRouteTable(): Promise<void> {
    const name = routeTableName.value.trim();
    if (!name) {
      onError('Route table name is required.');
      return;
    }

    savingRouteTable.value = true;

    try {
      await saveRouteTableApi(name, selectedTable.value ?? { openai: [], anthropic: [] });
      selectedRouteTable.value = name;
      await reload();
      applyRouteTable(name);
      onStatus('Route table saved.');
    } catch (error) {
      onError(error instanceof Error ? error.message : 'Failed to save route table.');
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
      onStatus(`Route table ${deleted} deleted.`);
    } catch (error) {
      onError(error instanceof Error ? error.message : 'Failed to delete route table.');
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
      onStatus('Route table activated.');
    } catch (error) {
      onError(error instanceof Error ? error.message : 'Failed to activate route table.');
    } finally {
      activatingRouteTable.value = false;
    }
  }

  return {
    activateRouteTable,
    activatingRouteTable,
    applyRouteTable,
    deleteSelectedRouteTable,
    deletingRouteTable,
    moveRouteInTable,
    reconcileRouteTableSelection,
    resetRouteTableForm,
    routeTableName,
    saveRouteTable,
    savingRouteTable,
    selectedRouteTable,
    selectedTable,
    toggleRouteInTable,
  };
}
