import { computed, ref } from 'vue';
import type { Protocol } from './types';

export type Locale = 'en' | 'zh';

type MessageValue = string | ((params: Record<string, string | number>) => string);
type Messages = Record<string, MessageValue>;

const STORAGE_KEY = 'yakumo-router-locale';

const messages: Record<Locale, Messages> = {
  en: {
    activate: 'Activate',
    activating: 'Activating...',
    active: 'Active',
    activeRouteTable: 'Active route table',
    apiKey: 'API Key',
    baseUrl: 'Base URL',
    configurationSummary: 'Configuration summary',
    createConfig: 'Create config',
    creatingConfig: 'Creating config...',
    contains: 'Contains',
    delete: 'Delete',
    deleting: 'Deleting...',
    editProvider: 'Edit Provider',
    exact: 'Exact',
    failedLoadConfig: 'Failed to load configuration.',
    failedSaveProvider: 'Failed to save provider.',
    failedDeleteProvider: 'Failed to delete provider.',
    failedSaveRule: 'Failed to save rule.',
    failedDeleteRule: 'Failed to delete rule.',
    failedSaveRouteTable: 'Failed to save route table.',
    failedDeleteRouteTable: 'Failed to delete route table.',
    failedActivateRouteTable: 'Failed to activate route table.',
    failedCreateConfig: 'Failed to create config file.',
    forwardOnly: 'Forward only',
    hideApiKey: 'Hide API key',
    inactive: 'Inactive',
    isActive: 'is active',
    isInactive: 'is inactive',
    language: 'Language',
    match: 'Match',
    matchType: 'Match Type',
    moveDown: 'Move down',
    moveUp: 'Move up',
    missingConfigDetail:
      'Create a minimal config.toml with only server and TLS settings before adding providers and rules.',
    missingConfigTitle: 'No config file found',
    name: 'Name',
    newProvider: 'New Provider',
    newProviderFor: ({ protocol }) => `New ${protocol} provider`,
    newRouteTable: 'New route table',
    newRule: 'New rule',
    noProviders: 'No providers configured.',
    noRouteTables: 'No route tables configured.',
    noRules: 'No rules configured.',
    noProviderRules: 'No rules for this provider.',
    none: 'None',
    providerCreated: 'Provider created.',
    providerDeleted: ({ count }) => `Provider deleted. ${count} route(s) removed.`,
    providerSaved: 'Provider saved.',
    providerSettings: ({ protocol }) => `${protocol} provider settings`,
    providers: 'Providers',
    providersSection: ({ protocol }) => `${protocol} Providers`,
    refresh: 'Refresh',
    refreshing: 'Refreshing',
    regex: 'Regex',
    routeDeleted: ({ id }) => `Rule ${id} deleted.`,
    routeTable: 'Route Table',
    routeTableActivated: 'Route table activated.',
    routeTableDeleted: ({ name }) => `Route table ${name} deleted.`,
    routeTableNameRequired: 'Route table name is required.',
    routeTableNewContext: 'Route table / new',
    routeTableSaved: 'Route table saved.',
    routeTables: 'Route Tables',
    routeTableContext: ({ name }) => `Route table / ${name}`,
    routesUsing: ({ provider }) => `Routes using ${provider}`,
    ruleCreated: 'Rule created.',
    ruleSaved: 'Rule saved.',
    rules: 'Rules',
    rulesSection: ({ protocol }) => `${protocol} Rules`,
    save: 'Save',
    saving: 'Saving...',
    selectProviderForRules: 'Select a provider to edit rules.',
    showApiKey: 'Show API key',
    tables: 'Tables',
    upstreamModel: 'Upstream Model',
  },
  zh: {
    activate: '激活',
    activating: '激活中...',
    active: '已激活',
    activeRouteTable: '当前激活的路由表',
    apiKey: 'API Key',
    baseUrl: 'Base URL',
    configurationSummary: '配置摘要',
    createConfig: '创建配置文件',
    creatingConfig: '创建中...',
    contains: '包含',
    delete: 'Delete',
    deleting: '删除中...',
    editProvider: '编辑 Provider',
    exact: '精确',
    failedLoadConfig: '加载配置失败。',
    failedSaveProvider: '保存 provider 失败。',
    failedDeleteProvider: '删除 provider 失败。',
    failedSaveRule: '保存规则失败。',
    failedDeleteRule: '删除规则失败。',
    failedSaveRouteTable: '保存路由表失败。',
    failedDeleteRouteTable: '删除路由表失败。',
    failedActivateRouteTable: '激活路由表失败。',
    failedCreateConfig: '创建配置文件失败。',
    forwardOnly: '仅转发',
    hideApiKey: '隐藏 API key',
    inactive: '未激活',
    isActive: '已激活',
    isInactive: '未激活',
    language: '语言',
    match: '匹配',
    matchType: '匹配类型',
    moveDown: '下移',
    moveUp: '上移',
    missingConfigDetail:
      '先创建一个只包含 server 和 tls 的最小 config.toml，然后再添加 provider 和规则。',
    missingConfigTitle: '未找到配置文件',
    name: '名称',
    newProvider: '新建 Provider',
    newProviderFor: ({ protocol }) => `新建 ${protocol} provider`,
    newRouteTable: '新建路由表',
    newRule: '新建规则',
    noProviders: '还没有配置 provider。',
    noRouteTables: '还没有配置路由表。',
    noRules: '还没有配置规则。',
    noProviderRules: '这个 provider 还没有规则。',
    none: '无',
    providerCreated: 'Provider 已创建。',
    providerDeleted: ({ count }) => `Provider 已删除，同时移除了 ${count} 条规则。`,
    providerSaved: 'Provider 已保存。',
    providerSettings: ({ protocol }) => `${protocol} provider 设置`,
    providers: 'Providers',
    providersSection: ({ protocol }) => `${protocol} Providers`,
    refresh: '刷新',
    refreshing: '刷新中',
    regex: '正则',
    routeDeleted: ({ id }) => `规则 ${id} 已删除。`,
    routeTable: '路由表',
    routeTableActivated: '路由表已激活。',
    routeTableDeleted: ({ name }) => `路由表 ${name} 已删除。`,
    routeTableNameRequired: '路由表名称不能为空。',
    routeTableNewContext: '路由表 / 新建',
    routeTableSaved: '路由表已保存。',
    routeTables: '路由表',
    routeTableContext: ({ name }) => `路由表 / ${name}`,
    routesUsing: ({ provider }) => `使用 ${provider} 的规则`,
    ruleCreated: '规则已创建。',
    ruleSaved: '规则已保存。',
    rules: '规则',
    rulesSection: ({ protocol }) => `${protocol} 规则`,
    save: 'Save',
    saving: '保存中...',
    selectProviderForRules: '选择一个 provider 后编辑规则。',
    showApiKey: '显示 API key',
    tables: '路由表',
    upstreamModel: '上游模型',
  },
};

export const localeOptions: { label: string; value: Locale }[] = [
  { label: 'English', value: 'en' },
  { label: '中文', value: 'zh' },
];

const storedLocale = window.localStorage.getItem(STORAGE_KEY);
const locale = ref<Locale>(storedLocale === 'zh' ? 'zh' : 'en');

export function protocolLabel(protocol: Protocol): string {
  return protocol === 'openai' ? 'OpenAI' : 'Anthropic';
}

export function useI18n() {
  const currentLocale = computed(() => locale.value);

  function setLocale(nextLocale: Locale): void {
    locale.value = nextLocale;
    window.localStorage.setItem(STORAGE_KEY, nextLocale);
  }

  function t(key: string, params: Record<string, string | number> = {}): string {
    const message = messages[locale.value][key] ?? messages.en[key] ?? key;
    return typeof message === 'function' ? message(params) : message;
  }

  return {
    locale: currentLocale,
    localeOptions,
    protocolLabel,
    setLocale,
    t,
  };
}
