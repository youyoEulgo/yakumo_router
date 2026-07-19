import { ref } from 'vue';
import type { Protocol } from '../types';

type RouteDragItem = {
  protocol: Protocol;
  routeId: string;
};

type DropTarget = RouteDragItem & {
  placement: 'before' | 'after';
};

type RouteDragSortOptions = {
  canDragRoute: (protocol: Protocol, routeId: string) => boolean;
  enabledRouteIds: (protocol: Protocol) => string[];
  moveRoute: (protocol: Protocol, routeId: string, direction: -1 | 1) => void;
};

export function useRouteDragSort({
  canDragRoute,
  enabledRouteIds,
  moveRoute,
}: RouteDragSortOptions) {
  const draggingRoute = ref<RouteDragItem | null>(null);
  const dropTarget = ref<DropTarget | null>(null);

  function onDragStart(protocol: Protocol, routeId: string, event: DragEvent): void {
    if (!canDragRoute(protocol, routeId)) {
      event.preventDefault();
      return;
    }

    draggingRoute.value = { protocol, routeId };
    event.dataTransfer?.setData('text/plain', routeId);
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
    }
  }

  function onDragOver(protocol: Protocol, routeId: string, event: DragEvent): void {
    if (
      !draggingRoute.value ||
      draggingRoute.value.protocol !== protocol ||
      draggingRoute.value.routeId === routeId ||
      !canDragRoute(protocol, routeId)
    ) {
      return;
    }

    event.preventDefault();
    const target = event.currentTarget as HTMLElement;
    const { top, height } = target.getBoundingClientRect();
    const placement = event.clientY < top + height / 2 ? 'before' : 'after';
    dropTarget.value = { protocol, routeId, placement };
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
  }

  function onDrop(protocol: Protocol, targetRouteId: string, event: DragEvent): void {
    event.preventDefault();
    const dragged = draggingRoute.value;
    if (!dragged || dragged.protocol !== protocol || dragged.routeId === targetRouteId) {
      clearDragState();
      return;
    }

    const ids = enabledRouteIds(protocol);
    const fromIndex = ids.indexOf(dragged.routeId);
    const targetIndex = ids.indexOf(targetRouteId);
    if (fromIndex === -1 || targetIndex === -1) {
      clearDragState();
      return;
    }

    const placement = dropTarget.value?.placement ?? 'before';
    const toIndex = placement === 'before' ? targetIndex : targetIndex + 1;
    const adjustedToIndex = fromIndex < toIndex ? toIndex - 1 : toIndex;
    if (fromIndex === adjustedToIndex) {
      clearDragState();
      return;
    }

    const direction: -1 | 1 = fromIndex < adjustedToIndex ? 1 : -1;
    for (let index = fromIndex; index !== adjustedToIndex; index += direction) {
      moveRoute(protocol, dragged.routeId, direction);
    }
    clearDragState();
  }

  function clearDragState(): void {
    draggingRoute.value = null;
    dropTarget.value = null;
  }

  function clearDropTarget(): void {
    dropTarget.value = null;
  }

  function isDragging(protocol: Protocol, routeId: string): boolean {
    return draggingRoute.value?.protocol === protocol && draggingRoute.value.routeId === routeId;
  }

  function dropPlacement(protocol: Protocol, routeId: string): 'before' | 'after' | null {
    if (dropTarget.value?.protocol !== protocol || dropTarget.value.routeId !== routeId) {
      return null;
    }

    return dropTarget.value.placement;
  }

  return {
    clearDragState,
    clearDropTarget,
    dropPlacement,
    isDragging,
    onDragOver,
    onDragStart,
    onDrop,
  };
}
