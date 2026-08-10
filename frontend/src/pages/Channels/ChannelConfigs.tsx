/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia 
 * @license        MIT (https://www.tokensbyte.ai/)
 */

import React, { useEffect, useState } from 'react';
import { Table, Button, Space, Tag, Modal, Form, Input, InputNumber, Switch, Segmented, message, Popconfirm, Card, Typography, AutoComplete, Grid, Tooltip, Progress, Select, Divider, TimePicker } from 'antd';
import MobileCardList, { MobileCard, CardRow, CardActions } from '../../components/MobileCardList';
import { PlusOutlined, EditOutlined, DeleteOutlined, SyncOutlined, ClearOutlined, StopOutlined, PlayCircleOutlined, SettingOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import dayjs from 'dayjs';
import request from '../../utils/request';
import { timedisplayOffsetSuffix } from '../../utils/timedisplay';
import useSettingsStore from '../../store/settings';
import { useThemeStore } from '../../store/theme';
import type { ChannelConfig, ChannelCategory, Upstream } from '../../types';
import ChannelCategoryManager from '../../components/Channels/ChannelCategoryManager';
import {
  formatQuotaLimitDisplay,
  parseQuotaLimitInput,
  getEffectiveChannelPeriodUsed,
  validateQuotaHierarchy,
  isFiniteQuotaLimit,
} from '../../utils/quotaPeriod';

const { Title, Text } = Typography;
const { useBreakpoint } = Grid;

function formatDailyResetSummary(
  hour: number,
  minute: number,
  cooldown: number,
  tzSuffix: string,
): string {
  const hh = String(Math.min(23, Math.max(0, hour))).padStart(2, '0');
  const mm = String(Math.min(59, Math.max(0, minute))).padStart(2, '0');
  const cool = Math.max(0, cooldown);
  const timeText = `刷新 ${hh}:${mm}${tzSuffix}`;
  return cool > 0 ? `${timeText} · 冷却 ${cool} 分钟` : timeText;
}

const ChannelConfigs: React.FC = () => {
  const { t } = useTranslation();
  const screens = useBreakpoint();
  const { themeMode } = useThemeStore();
  const isLight = themeMode === 'light';
  const { settings } = useSettingsStore();
  const quotaTz = settings?.site?.default_timezone || 'Asia/Shanghai';
  const [configs, setConfigs] = useState<ChannelConfig[]>([]);
  const [categories, setCategories] = useState<ChannelCategory[]>([]);
  const [loading, setLoading] = useState(true);
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [editingConfig, setEditingConfig] = useState<ChannelConfig | null>(null);
  const [upstreams, setUpstreams] = useState<{ id: number, name: string }[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [searchText, setSearchText] = useState('');
  const [statusFilter, setStatusFilter] = useState<number | 'all'>('all');
  const [categoryFilter, setCategoryFilter] = useState<number | 'all' | 'unclassified'>('all');
  const [enableQuota, setEnableQuota] = useState(false);
  const [isCategoryManagerVisible, setIsCategoryManagerVisible] = useState(false);
  const [dailyResetModalOpen, setDailyResetModalOpen] = useState(false);
  const [dailyResetDraft, setDailyResetDraft] = useState({ hour: 0, minute: 0, cooldown: 0 });
  const [form] = Form.useForm();

  const fetchConfigs = async () => {
    setLoading(true);
    try {
      const resp = await (request.get('/channel-configs') as unknown as Promise<{ data: ChannelConfig[] }>);
      setConfigs(resp.data || []);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const fetchCategories = async () => {
    try {
      const resp = await (request.get('/channel-categories') as any);
      setCategories(Array.isArray(resp) ? resp : (resp?.data || []));
    } catch (e) {
      console.error(e);
    }
  };

  const fetchUpstreams = async () => {
    try {
      const pResp = await (request.get('/upstreams') as unknown as Promise<Upstream[]>);
      setUpstreams(pResp || []);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    fetchConfigs();
    fetchCategories();
    fetchUpstreams();
  }, []);

  const resolveCategoryName = (categoryId?: number | null) => {
    if (!categoryId) return '';
    return categories.find(c => c.id === categoryId)?.name || '';
  };

  const activeCategories = categories.filter(c => c.is_active === 1 || c.is_active === true);

  const handleAdd = () => {
    setEditingConfig(null);
    setEnableQuota(false);
    form.resetFields();
    form.setFieldsValue({
      sort_order: 0,
      rate: 1.0,
      priority: 0,
      weight: 1,
      status: 1,
      category_id: null,
      quota_limit: -1,
      daily_quota_limit: -1,
      weekly_quota_limit: -1,
      monthly_quota_limit: -1,
      daily_reset_hour: 0,
      daily_reset_minute: 0,
      daily_reset_cooldown_minutes: 0,
    });
    setIsModalVisible(true);
  };

  const handleEdit = (record: ChannelConfig) => {
    setEditingConfig(record);
    const q = record.quota_limit ?? -1;
    const dq = record.daily_quota_limit ?? -1;
    const wq = record.weekly_quota_limit ?? -1;
    const mq = record.monthly_quota_limit ?? -1;
    setEnableQuota(q >= 0 || dq >= 0 || wq >= 0 || mq >= 0);
    form.resetFields();
    form.setFieldsValue({
      ...record,
      status: record.status ?? 1,
      category_id: record.category_id ?? null,
      quota_limit: record.quota_limit ?? -1,
      daily_quota_limit: record.daily_quota_limit ?? -1,
      weekly_quota_limit: record.weekly_quota_limit ?? -1,
      monthly_quota_limit: record.monthly_quota_limit ?? -1,
      daily_reset_hour: record.daily_reset_hour ?? 0,
      daily_reset_minute: record.daily_reset_minute ?? 0,
      daily_reset_cooldown_minutes: record.daily_reset_cooldown_minutes ?? 0,
    });
    setIsModalVisible(true);
  };

  const handleDelete = async (id: number) => {
    try {
      await request.delete(`/channel-configs/${id}`);
      message.success(t('common.success'));
      fetchConfigs();
    } catch (e) {
      console.error(e);
    }
  };

  const handleToggleStatus = async (record: ChannelConfig) => {
    try {
      const newStatus = (record.status ?? 1) === 1 ? 0 : 1;
      await request.put(`/channel-configs/${record.id}`, { status: newStatus });
      setConfigs(prev => prev.map(c => c.id === record.id ? { ...c, status: newStatus } : c));
      message.success(newStatus === 1 ? '已启用上游渠道' : '已禁用上游渠道');
    } catch (e) {
      console.error(e);
      message.error('状态更新失败');
    }
  };

  const handleResetQuota = async (id: number) => {
    try {
      await request.post(`/channel-configs/${id}/quota/reset`);
      message.success('已清零上游预设已用额度');
      fetchConfigs();
    } catch (e) {
      console.error(e);
      message.error('清零额度失败');
    }
  };

  const openDailyResetModal = () => {
    setDailyResetDraft({
      hour: Number(form.getFieldValue('daily_reset_hour') ?? 0),
      minute: Number(form.getFieldValue('daily_reset_minute') ?? 0),
      cooldown: Number(form.getFieldValue('daily_reset_cooldown_minutes') ?? 0),
    });
    setDailyResetModalOpen(true);
  };

  const saveDailyResetModal = () => {
    form.setFieldsValue({
      daily_reset_hour: Math.min(23, Math.max(0, Number(dailyResetDraft.hour) || 0)),
      daily_reset_minute: Math.min(59, Math.max(0, Number(dailyResetDraft.minute) || 0)),
      daily_reset_cooldown_minutes: Math.max(0, Number(dailyResetDraft.cooldown) || 0),
    });
    setDailyResetModalOpen(false);
  };

  const closeConfigModal = () => {
    setDailyResetModalOpen(false);
    setIsModalVisible(false);
  };

  const handleSave = async (values: any) => {
    if (submitting) return;
    setSubmitting(true);
    try {
      const payload = {
        ...values,
        base_url: values.base_url ? values.base_url.trim() : values.base_url,
        sort_order: Number(values.sort_order) || 0,
        rate: values.rate !== undefined && values.rate !== null ? Number(values.rate) : 1.0,
        priority: values.priority !== undefined && values.priority !== null ? Number(values.priority) : 0,
        weight: values.weight !== undefined && values.weight !== null ? Number(values.weight) : 1,
        status: values.status === 0 ? 0 : 1,
        category_id: values.category_id ?? null,
        quota_limit: (!enableQuota || values.quota_limit === undefined || values.quota_limit === null) ? -1 : Number(values.quota_limit),
        daily_quota_limit: (!enableQuota || values.daily_quota_limit === undefined || values.daily_quota_limit === null) ? -1 : Number(values.daily_quota_limit),
        weekly_quota_limit: (!enableQuota || values.weekly_quota_limit === undefined || values.weekly_quota_limit === null) ? -1 : Number(values.weekly_quota_limit),
        monthly_quota_limit: (!enableQuota || values.monthly_quota_limit === undefined || values.monthly_quota_limit === null) ? -1 : Number(values.monthly_quota_limit),
        daily_reset_hour: Math.min(23, Math.max(0, Number(values.daily_reset_hour) || 0)),
        daily_reset_minute: Math.min(59, Math.max(0, Number(values.daily_reset_minute) || 0)),
        daily_reset_cooldown_minutes: Math.max(0, Number(values.daily_reset_cooldown_minutes) || 0),
      };
      if (enableQuota) {
        const hierarchyErr = validateQuotaHierarchy(payload);
        if (hierarchyErr) {
          message.error(hierarchyErr);
          setSubmitting(false);
          return;
        }
      }
      if (editingConfig) {
        // 密钥未修改（与加载时原值相同）时或未填写时不提交，防止覆盖；但如果显式清空(等于空字符串)，则提交给后端处理
        if (payload.api_key === undefined || payload.api_key === editingConfig.api_key) {
          delete payload.api_key;
        } else if (typeof payload.api_key === 'string') {
          payload.api_key = payload.api_key.trim();
        }
        await request.put(`/channel-configs/${editingConfig.id}`, payload);
        message.success(t('common.success'));
      } else {
        if (typeof payload.api_key === 'string') {
          payload.api_key = payload.api_key.trim();
        }
        await request.post('/channel-configs', payload);
        message.success(t('common.success'));
      }
      setIsModalVisible(false);
      setDailyResetModalOpen(false);
      fetchConfigs();
    } catch (e) {
      console.error(e);
    } finally {
      setSubmitting(false);
    }
  };

  const quotaRingPercent = (used: number, limit: number) => {
    if (limit < 0) return 0;
    if (limit === 0) return used > 0 ? 100 : 0;
    return Math.min(100, Math.round((used / limit) * 100));
  };

  /** 总 / 月 / 周 / 日 使用不同蓝色区分 */
  const quotaRingBlue: Record<string, string> = {
    total: '#1d4ed8', // 深蓝
    month: '#2563eb',
    week: '#3b82f6',
    day: '#60a5fa', // 浅蓝
  };

  const renderQuotaCell = (record: ChannelConfig) => {
    const used = record.quota_used || 0;
    const limit = record.quota_limit ?? -1;
    const dailyLimit = record.daily_quota_limit ?? -1;
    const weeklyLimit = record.weekly_quota_limit ?? -1;
    const monthlyLimit = record.monthly_quota_limit ?? -1;
    const { dailyUsed, weeklyUsed, monthlyUsed } = getEffectiveChannelPeriodUsed(record, quotaTz);
    const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(6));

    const items = [
      { key: 'total', label: '总', used, limit },
      { key: 'month', label: '月', used: monthlyUsed, limit: monthlyLimit },
      { key: 'week', label: '周', used: weeklyUsed, limit: weeklyLimit },
      { key: 'day', label: '日', used: dailyUsed, limit: dailyLimit },
    ];
    const hasAnyConfigured = items.some((item) => item.limit >= 0);

    const slotStyle: React.CSSProperties = {
      width: 40,
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      gap: 2,
      cursor: 'default',
    };
    const labelStyle: React.CSSProperties = {
      fontSize: 10,
      color: isLight ? 'rgba(0,0,0,0.4)' : 'rgba(255,255,255,0.45)',
      lineHeight: 1,
    };

    return (
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(4, 40px)',
          gap: 4,
          alignItems: 'center',
          justifyContent: 'start',
          width: 172,
        }}
      >
        {items.map((item, index) => {
          const configured = item.limit >= 0;
          // 全无限：只在「总」位展示一个 ∞，其余占位保持对齐
          const showUnlimited = !hasAnyConfigured && index === 0;
          const showRing = configured || showUnlimited;

          if (!showRing) {
            return (
              <div key={item.key} style={{ ...slotStyle, visibility: 'hidden' }} aria-hidden>
                <div style={{ width: 36, height: 36 }} />
                <span style={labelStyle}>{item.label}</span>
              </div>
            );
          }

          const pct = configured ? quotaRingPercent(item.used, item.limit) : 0;
          const tip = showUnlimited
            ? `额度：${fmt(item.used)} / ∞（无限）`
            : `${item.label}额度：${fmt(item.used)} / ${fmt(Number(item.limit))}（${pct}%）`;
          const stroke = showUnlimited
            ? (isLight ? '#a1a1aa' : 'rgba(255,255,255,0.28)')
            : (pct >= 100 ? '#ef4444' : (quotaRingBlue[item.key] || '#3b82f6'));

          return (
            <Tooltip key={item.key} title={tip}>
              <div style={slotStyle}>
                <Progress
                  type="circle"
                  percent={showUnlimited ? 100 : pct}
                  size={36}
                  strokeWidth={10}
                  strokeColor={stroke}
                  trailColor={isLight ? '#e4e4e7' : 'rgba(255,255,255,0.12)'}
                  format={() => (
                    <span
                      style={{
                        fontSize: showUnlimited ? 11 : 10,
                        fontWeight: 600,
                        color: isLight ? 'rgba(0,0,0,0.72)' : 'rgba(255,255,255,0.88)',
                        lineHeight: 1,
                      }}
                    >
                      {showUnlimited ? '∞' : `${pct}%`}
                    </span>
                  )}
                />
                <span style={labelStyle}>{showUnlimited ? '无限' : item.label}</span>
              </div>
            </Tooltip>
          );
        })}
      </div>
    );
  };

  const renderStatusBadge = (status?: number) => {
    const active = (status ?? 1) === 1;
    return (
      <Space size={6} style={{ color: active ? '#52c41a' : '#ff4d4f' }}>
        <div style={{ width: 6, height: 6, borderRadius: '50%', backgroundColor: active ? '#52c41a' : '#ff4d4f' }} />
        <span style={{ fontSize: 13 }}>{active ? t('common.active') : t('common.disabled')}</span>
      </Space>
    );
  };

  const filteredConfigs = configs.filter(config => {
    if (statusFilter !== 'all' && (config.status ?? 1) !== statusFilter) {
      return false;
    }
    if (categoryFilter === 'unclassified') {
      if (config.category_id) return false;
    } else if (categoryFilter !== 'all') {
      if (config.category_id !== categoryFilter) return false;
    }
    if (searchText) {
      const searchLower = searchText.toLowerCase();
      const nameMatch = config.name?.toLowerCase().includes(searchLower);
      const yidMatch = config.yid?.toLowerCase().includes(searchLower);
      const providerMatch = config.provider_type?.toLowerCase().includes(searchLower);
      return nameMatch || yidMatch || providerMatch;
    }
    return true;
  });

  const columns = [
    {
      title: '配置',
      key: 'name',
      width: 180,
      ellipsis: true,
      render: (_: unknown, record: ChannelConfig) => (
        <div style={{ minWidth: 0 }}>
          <div style={{ fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{record.name}</div>
          <Typography.Text keyboard style={{ color: '#1677ff', fontSize: 11 }}>
            YID {record.yid || '-'}
          </Typography.Text>
        </div>
      ),
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 90,
      render: (status: number) => renderStatusBadge(status),
    },
    {
      title: '上游分类',
      dataIndex: 'category_id',
      key: 'category_id',
      width: 100,
      ellipsis: true,
      render: (categoryId: number | null | undefined) => {
        const name = resolveCategoryName(categoryId);
        return name ? <Tag style={{ margin: 0 }}>{name}</Tag> : <Text type="secondary">未分类</Text>;
      },
    },
    {
      title: '服务商',
      dataIndex: 'provider_type',
      key: 'provider_type',
      width: 100,
      ellipsis: true,
      render: (text: string) => text || '-',
    },
    {
      title: '调度',
      key: 'schedule',
      width: 110,
      render: (_: unknown, record: ChannelConfig) => (
        <Space size={4} wrap={false}>
          <Tooltip title="优先级"><Text type="secondary" style={{ fontSize: 12 }}>P{record.priority || 0}</Text></Tooltip>
          <Tooltip title="权重"><Text type="secondary" style={{ fontSize: 12 }}>W{record.weight || 1}</Text></Tooltip>
          <Tag color="orange" style={{ margin: 0, lineHeight: '18px', fontSize: 12 }}>{record.rate ?? 1.0}x</Tag>
        </Space>
      ),
    },
    {
      title: '额度',
      key: 'quota',
      width: 188,
      render: (_: unknown, record: ChannelConfig) => renderQuotaCell(record),
    },
    {
      title: 'Base URL',
      dataIndex: 'base_url',
      key: 'base_url',
      width: 200,
      ellipsis: true,
      render: (text: string) => (
        <Tooltip title={text}>
          <Text code style={{ fontSize: 12 }}>{text}</Text>
        </Tooltip>
      ),
    },
    {
      title: '排序',
      dataIndex: 'sort_order',
      key: 'sort_order',
      width: 56,
      align: 'center' as const,
      render: (val: number) => <Text type="secondary">{val || 0}</Text>,
    },
    {
      title: '备注',
      dataIndex: 'remark',
      key: 'remark',
      width: 120,
      ellipsis: true,
      render: (text: string) => <Text type="secondary">{text || '-'}</Text>,
    },
    {
      title: t('common.actions'),
      key: 'actions',
      width: 150,
      fixed: 'right' as const,
      render: (_: unknown, record: ChannelConfig) => (
        <Space size={0}>
          <Tooltip title={(record.status ?? 1) === 1 ? '点击禁用' : '点击启用'}>
            <Button
              type="text"
              size="small"
              icon={(record.status ?? 1) === 1
                ? <PlayCircleOutlined style={{ color: '#52c41a' }} />
                : <StopOutlined style={{ color: '#ff4d4f' }} />}
              onClick={() => handleToggleStatus(record)}
            />
          </Tooltip>
          <Tooltip title="清零额度">
            <Popconfirm title="确定清零该预设的总/日/周/月已用额度吗？" onConfirm={() => handleResetQuota(record.id)}>
              <Button type="text" size="small" icon={<ClearOutlined />} />
            </Popconfirm>
          </Tooltip>
          <Tooltip title="编辑">
            <Button type="text" size="small" icon={<EditOutlined />} onClick={() => handleEdit(record)} />
          </Tooltip>
          <Tooltip title="删除">
            <Popconfirm title={t('common.confirm_delete')} onConfirm={() => handleDelete(record.id)}>
              <Button type="text" size="small" icon={<DeleteOutlined />} danger />
            </Popconfirm>
          </Tooltip>
        </Space>
      ),
    },
  ];

  return (
    <Card bordered={false}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: 16, marginBottom: 24 }}>
        <Title level={4} style={{ margin: 0, fontSize: screens.xs ? 18 : 20, fontWeight: 600 }}>上游渠道配置预设</Title>
        <Space wrap>
          <Segmented
            options={[
              { label: `全部 (${configs.length})`, value: 'all' },
              { label: `激活 (${configs.filter(c => (c.status ?? 1) === 1).length})`, value: '1' },
              { label: `已禁用 (${configs.filter(c => (c.status ?? 1) === 0).length})`, value: '0' },
            ]}
            value={statusFilter === 'all' ? 'all' : statusFilter.toString()}
            onChange={(val) => setStatusFilter(val === 'all' ? 'all' : parseInt(val as string, 10))}
          />
          <Input.Search
            placeholder="名称、YID或服务商"
            allowClear
            onSearch={setSearchText}
            onChange={(e) => !e.target.value && setSearchText('')}
            style={{ width: screens.xs ? '100%' : 220 }}
          />
          <Button icon={<SyncOutlined />} onClick={fetchConfigs}>刷新</Button>
          <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>添加配置</Button>
        </Space>
      </div>

      <div style={{
        backgroundColor: isLight ? '#fafafa' : '#141414',
        padding: '12px 16px',
        borderRadius: 8,
        marginBottom: 16,
        border: isLight ? '1px solid #e8e8e8' : '1px solid #303030',
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', padding: 0 }}>
          <Text type="secondary" style={{ width: 80, flexShrink: 0, fontSize: 13 }}>上游分类</Text>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, flexGrow: 1 }}>
            {([
              { key: 'all' as const, label: '全部', count: configs.length },
              ...categories.map(c => ({
                key: c.id,
                label: c.name,
                count: configs.filter(ch => ch.category_id === c.id).length,
              })),
              {
                key: 'unclassified' as const,
                label: '未分类',
                count: configs.filter(ch => !ch.category_id).length,
              },
            ] as { key: number | 'all' | 'unclassified'; label: string; count: number }[]).map(item => {
              const selected = categoryFilter === item.key;
              return (
                <div
                  key={String(item.key)}
                  onClick={() => setCategoryFilter(item.key)}
                  style={{
                    padding: '4px 12px',
                    borderRadius: 16,
                    fontSize: 14,
                    backgroundColor: selected ? '#1677ff' : (isLight ? '#f0f0f0' : '#1d1d1d'),
                    color: selected ? '#fff' : (isLight ? 'rgba(0,0,0,0.65)' : 'rgba(255,255,255,0.65)'),
                    border: isLight ? '1px solid #d9d9d9' : '1px solid #303030',
                    cursor: 'pointer',
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: 6,
                    transition: 'all 0.2s',
                  }}
                >
                  {item.label}
                  <span style={{ opacity: 0.6 }}>{item.count}</span>
                </div>
              );
            })}
            <Tooltip title={t('common.manage', '管理')}>
              <Button
                type="text"
                size="small"
                icon={<SettingOutlined style={{ color: '#1677ff' }} />}
                onClick={() => setIsCategoryManagerVisible(true)}
                style={{ marginLeft: 8 }}
              />
            </Tooltip>
          </div>
        </div>
      </div>

      {screens.xs ? (
        <MobileCardList
          dataSource={filteredConfigs}
          loading={loading}
          rowKey="id"
          renderCard={(record: ChannelConfig) => (
            <MobileCard
              title={record.name}
              extra={<Typography.Text keyboard style={{ color: '#1677ff' }}>{record.yid || '-'}</Typography.Text>}
            >
              <CardRow label="状态">{renderStatusBadge(record.status)}</CardRow>
              <CardRow label="上游分类">{resolveCategoryName(record.category_id) || '未分类'}</CardRow>
              <CardRow label="服务商广场展示">{record.provider_type || '-'}</CardRow>
              <CardRow label="Base URL"><Text code style={{ fontSize: 12 }}>{record.base_url}</Text></CardRow>
              <CardRow label="额度">{renderQuotaCell(record)}</CardRow>
              <CardRow label="备注">{record.remark || '-'}</CardRow>
              <CardActions>
                <Tooltip title={(record.status ?? 1) === 1 ? '点击禁用' : '点击启用'}>
                  <Button
                    type="text"
                    size="small"
                    icon={(record.status ?? 1) === 1
                      ? <PlayCircleOutlined style={{ color: '#52c41a' }} />
                      : <StopOutlined style={{ color: '#ff4d4f' }} />}
                    onClick={() => handleToggleStatus(record)}
                  />
                </Tooltip>
                <Tooltip title="清零额度">
                  <Popconfirm title="确定清零已用额度吗？" onConfirm={() => handleResetQuota(record.id)}>
                    <Button type="text" size="small" icon={<ClearOutlined />} />
                  </Popconfirm>
                </Tooltip>
                <Button type="text" size="small" icon={<EditOutlined />} onClick={() => handleEdit(record)} />
                <Popconfirm title={t('common.confirm_delete')} onConfirm={() => handleDelete(record.id)}>
                  <Button type="text" size="small" icon={<DeleteOutlined />} danger />
                </Popconfirm>
              </CardActions>
            </MobileCard>
          )}
        />
      ) : (
        <Table
          size="small"
          dataSource={filteredConfigs}
          columns={columns}
          rowKey="id"
          loading={loading}
          pagination={{ pageSize: 15, showTotal: (total) => `共 ${total} 条` }}
          scroll={{ x: 1080 }}
        />
      )}

      <Modal
        title={editingConfig ? "编辑上游渠道配置" : "添加上游渠道配置"}
        open={isModalVisible}
        onCancel={closeConfigModal}
        onOk={() => form.submit()}
        confirmLoading={submitting}
        width={560}
        styles={{ body: { paddingTop: 12, paddingBottom: 4 } }}
      >
        <Form
          form={form}
          layout="vertical"
          size="small"
          onFinish={handleSave}
          autoComplete="off"
          style={{ marginBottom: 0 }}
        >
          <div style={{ display: 'flex', gap: 12 }}>
            <Form.Item
              name="name"
              label={
                <Space size={6}>
                  <span>配置名称</span>
                  {editingConfig?.yid && (
                    <Text type="secondary" style={{ fontSize: 11 }}>YID: {editingConfig.yid}</Text>
                  )}
                </Space>
              }
              rules={[{ required: true, message: '请输入配置名称' }]}
              style={{ flex: 1.4, marginBottom: 10 }}
            >
              <Input placeholder="例如：OpenAI 官方渠道" autoComplete="off" />
            </Form.Item>
            <Form.Item
              name="provider_type"
              label="服务商类型(模型广场展示)"
              style={{ flex: 1, marginBottom: 10 }}
            >
              <Input placeholder="如: custom / openai" autoComplete="off" />
            </Form.Item>
          </div>

          <div style={{ display: 'flex', gap: 12, alignItems: 'flex-start' }}>
            <Form.Item name="category_id" label="上游分类" style={{ flex: 1, marginBottom: 10 }}>
              <Select
                allowClear
                placeholder="选择分类"
                options={activeCategories.map(c => ({ label: c.name, value: c.id }))}
                dropdownRender={(menu) => (
                  <>
                    {menu}
                    <Divider style={{ margin: '6px 0' }} />
                    <Button
                      type="link"
                      size="small"
                      icon={<SettingOutlined />}
                      onClick={() => setIsCategoryManagerVisible(true)}
                      style={{ width: '100%' }}
                    >
                      管理分类
                    </Button>
                  </>
                )}
              />
            </Form.Item>
            <Form.Item
              name="sort_order"
              label={
                <Tooltip title="数字越大在页面中越靠前">
                  <span>页面排序</span>
                </Tooltip>
              }
              style={{ width: 96, marginBottom: 10 }}
            >
              <InputNumber placeholder="0" style={{ width: '100%' }} />
            </Form.Item>
          </div>

          <div style={{ display: 'flex', gap: 12 }}>
            <Form.Item name="rate" label="渠道倍率" rules={[{ required: true }]} style={{ flex: 1, marginBottom: 10 }}>
              <InputNumber min={0} step={0.1} placeholder="1.0" style={{ width: '100%' }} />
            </Form.Item>
            <Form.Item name="priority" label="优先级" rules={[{ required: true }]} style={{ flex: 1, marginBottom: 10 }}>
              <InputNumber min={0} placeholder="0" style={{ width: '100%' }} />
            </Form.Item>
            <Form.Item name="weight" label="权重" rules={[{ required: true }]} style={{ flex: 1, marginBottom: 10 }}>
              <InputNumber min={1} placeholder="1" style={{ width: '100%' }} />
            </Form.Item>
          </div>

          <Form.Item label="额度限额" style={{ marginBottom: 10 }}>
            <Switch
              checked={enableQuota}
              onChange={(checked) => {
                setEnableQuota(checked);
                form.setFieldsValue({
                  quota_limit: -1,
                  daily_quota_limit: -1,
                  weekly_quota_limit: -1,
                  monthly_quota_limit: -1,
                });
              }}
              checkedChildren="已开启限额"
              unCheckedChildren="默认不限额"
            />
          </Form.Item>

          <Form.Item name="daily_reset_hour" hidden initialValue={0}><InputNumber /></Form.Item>
          <Form.Item name="daily_reset_minute" hidden initialValue={0}><InputNumber /></Form.Item>
          <Form.Item name="daily_reset_cooldown_minutes" hidden initialValue={0}><InputNumber /></Form.Item>

          {enableQuota && (
            <Form.Item
              noStyle
              shouldUpdate={(prev, curr) =>
                prev.quota_limit !== curr.quota_limit ||
                prev.daily_quota_limit !== curr.daily_quota_limit ||
                prev.weekly_quota_limit !== curr.weekly_quota_limit ||
                prev.monthly_quota_limit !== curr.monthly_quota_limit ||
                prev.daily_reset_hour !== curr.daily_reset_hour ||
                prev.daily_reset_minute !== curr.daily_reset_minute ||
                prev.daily_reset_cooldown_minutes !== curr.daily_reset_cooldown_minutes
              }
            >
              {() => {
                const quotaDeps = ['quota_limit', 'daily_quota_limit', 'weekly_quota_limit', 'monthly_quota_limit'] as const;
                const fieldValidator = (field: typeof quotaDeps[number]) => ({
                  validator: async (_: unknown, value: number | null) => {
                    if (value != null && Number(value) < -1) {
                      throw new Error('额度不能小于 -1');
                    }
                    const latest = {
                      ...form.getFieldsValue([...quotaDeps]),
                      [field]: value,
                    };
                    const total = latest.quota_limit;
                    const day = latest.daily_quota_limit;
                    const week = latest.weekly_quota_limit;
                    const month = latest.monthly_quota_limit;

                    const fail = (msg: string) => {
                      throw new Error(msg);
                    };

                    if (field === 'daily_quota_limit' && isFiniteQuotaLimit(day)) {
                      if (isFiniteQuotaLimit(week) && day > week) fail('日额度不能大于周额度');
                      if (isFiniteQuotaLimit(month) && day > month) fail('日额度不能大于月额度');
                      if (isFiniteQuotaLimit(total) && day > total) fail('日额度不能大于总额度');
                    }
                    if (field === 'weekly_quota_limit' && isFiniteQuotaLimit(week)) {
                      if (isFiniteQuotaLimit(month) && week > month) fail('周额度不能大于月额度');
                      if (isFiniteQuotaLimit(total) && week > total) fail('周额度不能大于总额度');
                      if (isFiniteQuotaLimit(day) && day > week) fail('周额度不能小于日额度');
                    }
                    if (field === 'monthly_quota_limit' && isFiniteQuotaLimit(month)) {
                      if (isFiniteQuotaLimit(total) && month > total) fail('月额度不能大于总额度');
                      if (isFiniteQuotaLimit(week) && week > month) fail('月额度不能小于周额度');
                      if (isFiniteQuotaLimit(day) && day > month) fail('月额度不能小于日额度');
                    }
                    if (field === 'quota_limit' && isFiniteQuotaLimit(total)) {
                      if (isFiniteQuotaLimit(month) && month > total) fail('总额度不能小于月额度');
                      if (isFiniteQuotaLimit(week) && week > total) fail('总额度不能小于周额度');
                      if (isFiniteQuotaLimit(day) && day > total) fail('总额度不能小于日额度');
                    }
                  },
                });
                const quotaItemStyle: React.CSSProperties = { marginBottom: 10 };
                const resetHour = Number(form.getFieldValue('daily_reset_hour') ?? 0);
                const resetMinute = Number(form.getFieldValue('daily_reset_minute') ?? 0);
                const resetCooldown = Number(form.getFieldValue('daily_reset_cooldown_minutes') ?? 0);
                const tzSuffix = timedisplayOffsetSuffix(quotaTz);
                const resetSummary = formatDailyResetSummary(resetHour, resetMinute, resetCooldown, tzSuffix);
                return (
                  <div style={{ marginBottom: 4 }}>
                    <div
                      style={{
                        display: 'grid',
                        gridTemplateColumns: '1fr 1fr',
                        columnGap: 12,
                      }}
                    >
                      <Form.Item
                        name="quota_limit"
                        label="总额度"
                        initialValue={-1}
                        style={quotaItemStyle}
                        dependencies={quotaDeps as unknown as string[]}
                        validateTrigger={['onChange', 'onBlur']}
                        rules={[fieldValidator('quota_limit')]}
                      >
                        <InputNumber
                          min={-1}
                          style={{ width: '100%' }}
                          formatter={formatQuotaLimitDisplay}
                          parser={parseQuotaLimitInput}
                        />
                      </Form.Item>
                      <Form.Item
                        name="monthly_quota_limit"
                        label="月额度"
                        initialValue={-1}
                        style={quotaItemStyle}
                        dependencies={quotaDeps as unknown as string[]}
                        validateTrigger={['onChange', 'onBlur']}
                        rules={[fieldValidator('monthly_quota_limit')]}
                      >
                        <InputNumber
                          min={-1}
                          style={{ width: '100%' }}
                          formatter={formatQuotaLimitDisplay}
                          parser={parseQuotaLimitInput}
                        />
                      </Form.Item>
                      <Form.Item
                        name="weekly_quota_limit"
                        label="周额度"
                        initialValue={-1}
                        style={quotaItemStyle}
                        dependencies={quotaDeps as unknown as string[]}
                        validateTrigger={['onChange', 'onBlur']}
                        rules={[fieldValidator('weekly_quota_limit')]}
                      >
                        <InputNumber
                          min={-1}
                          style={{ width: '100%' }}
                          formatter={formatQuotaLimitDisplay}
                          parser={parseQuotaLimitInput}
                        />
                      </Form.Item>
                      <Form.Item
                        name="daily_quota_limit"
                        label="日额度"
                        initialValue={-1}
                        style={quotaItemStyle}
                        dependencies={quotaDeps as unknown as string[]}
                        validateTrigger={['onChange', 'onBlur']}
                        rules={[fieldValidator('daily_quota_limit')]}
                      >
                        <InputNumber
                          min={-1}
                          style={{ width: '100%' }}
                          formatter={formatQuotaLimitDisplay}
                          parser={parseQuotaLimitInput}
                        />
                      </Form.Item>
                    </div>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'flex-start',
                        gap: 10,
                        marginBottom: 10,
                        padding: '8px 12px',
                        borderRadius: 8,
                        border: isLight ? '1px solid rgba(0,0,0,0.06)' : '1px solid rgba(255,255,255,0.08)',
                        background: isLight ? 'rgba(0,0,0,0.02)' : 'rgba(255,255,255,0.04)',
                      }}
                    >
                      <Text strong style={{ fontSize: 12, flexShrink: 0, lineHeight: '22px' }}>日额度刷新</Text>
                      <Text
                        type="secondary"
                        title={resetSummary}
                        style={{ fontSize: 12, flex: 1, minWidth: 0, lineHeight: '22px', wordBreak: 'break-word' }}
                      >
                        {resetSummary}
                      </Text>
                      <Button
                        type="link"
                        size="small"
                        icon={<SettingOutlined />}
                        onClick={openDailyResetModal}
                        style={{ paddingInline: 0, flexShrink: 0, height: 22 }}
                      >
                        配置
                      </Button>
                    </div>
                  </div>
                );
              }}
            </Form.Item>
          )}

          <Form.Item name="base_url" label="端点基础地址 (Base URL)" rules={[{ required: true }]} style={{ marginBottom: 10 }}>
            <AutoComplete
              options={[
                { value: 'https://ark.cn-beijing.volces.com', label: '火山方舟 (https://ark.cn-beijing.volces.com)' },
                { value: 'https://ark.ap-southeast.bytepluses.com/api/v3', label: 'BytePlus(ap-southeast-1) (https://ark.ap-southeast.bytepluses.com/api/v3)' },
                { value: 'https://ark.eu-west.bytepluses.com/api/v3', label: 'BytePlus(eu-west-1) (https://ark.eu-west.bytepluses.com/api/v3)' },
                { value: 'https://api-beijing.klingai.com', label: '可灵 (https://api-beijing.klingai.com)' },
                { value: 'https://dashscope.aliyuncs.com', label: '阿里百炼 (https://dashscope.aliyuncs.com)' },
                { value: 'https://vod.tencentcloudapi.com', label: '腾讯云 VOD AIGC (https://vod.tencentcloudapi.com)' },
                { value: 'https://visual.volcengineapi.com', label: '即梦AI/火山CV (https://visual.volcengineapi.com)' },
              ]}
              placeholder="选择预设或自由输入"
              filterOption={(inputValue, option) =>
                String(option?.label || '').toUpperCase().indexOf(inputValue.toUpperCase()) !== -1 ||
                String(option?.value || '').toUpperCase().indexOf(inputValue.toUpperCase()) !== -1
              }
            />
          </Form.Item>
          <Form.Item
            name="api_key"
            label={
              <Tooltip
                title={
                  editingConfig
                    ? '保持不变直接保存即可，输入新值将覆盖旧密钥'
                    : '可灵新协议(kling_video)：官方 API Key 直传 Bearer；可灵旧协议(kling)：access_key:secret_key（自动 JWT）；腾讯云 VOD：SecretId:SecretKey:SubAppId；即梦AI：AccessKeyID:SecretAccessKey；其他：sk-xxx'
                }
              >
                <span>请求鉴权密钥 (API Key)</span>
              </Tooltip>
            }
            style={{ marginBottom: 10 }}
          >
            <Input.Password
              autoComplete="new-password"
              placeholder={editingConfig ? '保持当前密钥或输入新值覆盖' : 'API Key / sk-... / access_key:secret_key'}
            />
          </Form.Item>
          <Form.Item
            name="remark"
            label={
              <Tooltip title="记录渠道归属、适用场景等，方便查阅">
                <span>备注说明</span>
              </Tooltip>
            }
            style={{ marginBottom: 10 }}
          >
            <Input.TextArea rows={1} placeholder="例如：图片生成主通道..." autoSize={{ minRows: 1, maxRows: 3 }} />
          </Form.Item>
          <Form.Item
            name="status"
            label="状态"
            initialValue={1}
            valuePropName="checked"
            getValueFromEvent={(checked: boolean) => (checked ? 1 : 0)}
            getValueProps={(value) => ({ checked: value !== 0 })}
            style={{ marginBottom: 0 }}
          >
            <Switch checkedChildren={t('common.active')} unCheckedChildren={t('common.disabled')} />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title="日额度刷新设置"
        open={dailyResetModalOpen}
        onCancel={() => setDailyResetModalOpen(false)}
        onOk={saveDailyResetModal}
        okText="保存"
        cancelText="取消"
        width={360}
        destroyOnHidden
        getContainer={() => document.body}
        zIndex={1100}
      >
        <Form layout="vertical" size="small" style={{ marginTop: 8 }}>
          <Form.Item
            label={`每天刷新时间点${timedisplayOffsetSuffix(quotaTz)}`}
            style={{ marginBottom: 12 }}
          >
            <TimePicker
              format="HH:mm"
              allowClear={false}
              style={{ width: '100%' }}
              getPopupContainer={(node) => node.parentElement || document.body}
              value={dayjs().hour(dailyResetDraft.hour).minute(dailyResetDraft.minute).second(0)}
              onChange={(t) => {
                setDailyResetDraft((prev) => ({
                  ...prev,
                  hour: t ? t.hour() : 0,
                  minute: t ? t.minute() : 0,
                }));
              }}
            />
          </Form.Item>
          <Form.Item
            label="刷新冷却（分钟）"
            extra="到达时间点后再等待该时长才清零日已用"
            style={{ marginBottom: 8 }}
          >
            <InputNumber
              min={0}
              max={1440}
              placeholder="0"
              style={{ width: '100%' }}
              addonAfter="分钟"
              value={dailyResetDraft.cooldown}
              onChange={(v) => setDailyResetDraft((prev) => ({
                ...prev,
                cooldown: v == null ? 0 : Number(v),
              }))}
            />
          </Form.Item>
          <Text type="secondary" style={{ fontSize: 11 }}>
            按站点默认时区计算（非系统运行时区）；默认 00:00 / 冷却 0 即自然日刷新
          </Text>
        </Form>
      </Modal>

      <ChannelCategoryManager
        visible={isCategoryManagerVisible}
        onClose={() => setIsCategoryManagerVisible(false)}
        onUpdate={fetchCategories}
      />
    </Card>
  );
};

export default ChannelConfigs;
