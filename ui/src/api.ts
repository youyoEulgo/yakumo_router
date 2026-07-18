import type {
  ConfigState,
  DeleteProviderResult,
  DeleteRouteResult,
  ProviderConfig,
  ProviderTables,
  Protocol,
  RouteRule,
  RouteTables,
  RouteTable,
  RouteTableState,
  UpsertProviderResult,
  UpsertRouteResult,
} from './types';

async function readJson<T>(response: Response, label: string): Promise<T> {
  if (!response.ok) {
    throw new Error(`${label} failed with ${response.status}`);
  }

  return (await response.json()) as T;
}

export async function loadConfigState(): Promise<ConfigState> {
  const [providersResponse, routesResponse, routeTablesResponse] = await Promise.all([
    fetch('/_ui/api/providers'),
    fetch('/_ui/api/routes'),
    fetch('/_ui/api/route-tables'),
  ]);

  const providers = await readJson<ProviderTables>(providersResponse, 'Providers request');
  const routes = await readJson<RouteTables>(routesResponse, 'Routes request');
  const routeTables = await readJson<RouteTableState>(
    routeTablesResponse,
    'Route tables request',
  );

  return {
    providers: {
      openai: providers.openai ?? {},
      anthropic: providers.anthropic ?? {},
    },
    routes: {
      openai: routes.openai ?? [],
      anthropic: routes.anthropic ?? [],
    },
    routeTables: {
      active: routeTables.active ?? null,
      tables: routeTables.tables ?? {},
    },
  };
}

export async function saveProvider(
  protocol: Protocol,
  name: string,
  provider: ProviderConfig,
): Promise<UpsertProviderResult> {
  const response = await fetch(`/_ui/api/providers/${protocol}/${name}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(provider),
  });

  return readJson<UpsertProviderResult>(response, 'Provider save');
}

export async function deleteProvider(
  protocol: Protocol,
  name: string,
): Promise<DeleteProviderResult> {
  const response = await fetch(`/_ui/api/providers/${protocol}/${name}`, {
    method: 'DELETE',
  });

  return readJson<DeleteProviderResult>(response, 'Provider delete');
}

export async function saveRoute(
  protocol: Protocol,
  route: RouteRule,
): Promise<UpsertRouteResult> {
  const response = await fetch(`/_ui/api/routes/${protocol}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(route),
  });

  return readJson<UpsertRouteResult>(response, 'Route save');
}

export async function deleteRoute(protocol: Protocol, id: string): Promise<DeleteRouteResult> {
  const response = await fetch(`/_ui/api/routes/${protocol}/${id}`, {
    method: 'DELETE',
  });

  return readJson<DeleteRouteResult>(response, 'Rule delete');
}

export async function saveRouteTable(name: string, table: RouteTable): Promise<void> {
  const response = await fetch(`/_ui/api/route-tables/${name}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(table),
  });

  await readJson(response, 'Route table save');
}

export async function deleteRouteTable(name: string): Promise<void> {
  const response = await fetch(`/_ui/api/route-tables/${name}`, {
    method: 'DELETE',
  });

  await readJson(response, 'Route table delete');
}

export async function activateRouteTable(name: string): Promise<void> {
  const response = await fetch(`/_ui/api/active-route-table/${name}`, {
    method: 'PUT',
  });

  await readJson(response, 'Route table activation');
}
