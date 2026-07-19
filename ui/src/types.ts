export type Protocol = 'openai' | 'anthropic';
export type MatchType = 'contains' | 'exact' | 'regex';
export type EditorPane = 'provider' | 'route-table';

export interface ProviderConfig {
  base_url: string;
  api_key: string;
}

export interface RouteRule {
  id: string;
  match: string;
  match_type: MatchType;
  provider: string;
  model: string;
  forward_only: boolean;
}

export interface ProviderTables {
  openai: Record<string, ProviderConfig>;
  anthropic: Record<string, ProviderConfig>;
}

export interface RouteTables {
  openai: RouteRule[];
  anthropic: RouteRule[];
}

export interface RouteTable {
  openai: string[];
  anthropic: string[];
}

export interface RouteTableState {
  active: string | null;
  tables: Record<string, RouteTable>;
}

export interface ConfigState {
  providers: ProviderTables;
  routes: RouteTables;
  routeTables: RouteTableState;
}

export interface UpsertRouteResult {
  updated: boolean;
  route: RouteRule;
}

export interface UpsertProviderResult {
  updated: boolean;
  name: string;
  provider: ProviderConfig;
}

export interface DeleteProviderResult {
  name: string;
  removed_routes: number;
}

export interface DeleteRouteResult {
  id: string;
}
