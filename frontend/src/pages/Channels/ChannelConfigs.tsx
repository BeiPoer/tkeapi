/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia 
 * @license        MIT (https://www.tokensbyte.ai/)
 */

import React, { useEffect, useState } from 'react';
import { Table, Button, Space, Tag, Modal, Form, Input, InputNumber, Switch, Segmented, message, Popconfirm, Card, Typography, AutoComplete, Grid, Tooltip, Progress, Select, Divider } from 'antd';
import MobileCardList, { MobileCard, CardRow, CardActions } from '../../components/MobileCardList';
import { PlusOutlined, EditOutlined, DeleteOutlined, SyncOutlined, ClearOutlined, StopOutlined, PlayCircleOutlined, SettingOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import request from '../../utils/request';
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
        onCancel={() => setIsModalVisible(false)}
        onOk={() => form.submit()}
        confirmLoading={submitting}
        width={640}
      >
        <Form form={form} layout="vertical" onFinish={handleSave} autoComplete="off">
          <Form.Item name="name" label={
            <Space>
              <span>配置名称</span>
              {editingConfig?.yid && (
                <Text type="secondary" style={{ fontSize: 12 }}>YID: {editingConfig.yid}</Text>
              )}
            </Space>
          } rules={[{ required: true }]}>
            <Input placeholder="例如：OpenAI 官方渠道接口" autoComplete="off" />
          </Form.Item>
          <Form.Item name="provider_type" label="服务商类型(模型广场展示)">
            <Input placeholder="可自由输入 (如: custom)" autoComplete="off" />
          </Form.Item>
          <Form.Item name="category_id" label="上游分类">
            <Select
              allowClear
              placeholder="选择分类"
              options={activeCategories.map(c => ({ label: c.name, value: c.id }))}
              dropdownRender={(menu) => (
                <>
                  {menu}
                  <Divider style={{ margin: '8px 0' }} />
                  <Button
                    type="link"
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
          <div style={{ display: 'flex', gap: '16px' }}>
            <Form.Item name="rate" label="渠道倍率" rules={[{ required: true }]} style={{ flex: 1 }}>
              <InputNumber min={0} step={0.1} placeholder="1.0" style={{ width: '100%' }} />
            </Form.Item>
            <Form.Item name="priority" label="优先级" rules={[{ required: true }]} style={{ flex: 1 }}>
              <InputNumber min={0} placeholder="0" style={{ width: '100%' }} />
            </Form.Item>
            <Form.Item name="weight" label="权重" rules={[{ required: true }]} style={{ flex: 1 }}>
              <InputNumber min={1} placeholder="1" style={{ width: '100%' }} />
            </Form.Item>
          </div>
          <Form.Item label="额度配置" style={{ marginBottom: 16 }}>
            <Switch 
              checked={enableQuota}
              onChange={(checked) => {
                setEnableQuota(checked);
                if (!checked) {
                  form.setFieldsValue({
                    quota_limit: -1,
                    daily_quota_limit: -1,
                    weekly_quota_limit: -1,
                    monthly_quota_limit: -1,
                  });
                } else {
                  form.setFieldsValue({
                    quota_limit: -1,
                    daily_quota_limit: -1,
                    weekly_quota_limit: -1,
                    monthly_quota_limit: -1,
                  });
                }
              }}
              checkedChildren="已开启限额"
              unCheckedChildren="默认不限额"
            />
          </Form.Item>
          {enableQuota && (
            <Form.Item
              noStyle
              shouldUpdate={(prev, curr) =>
                prev.quota_limit !== curr.quota_limit ||
                prev.daily_quota_limit !== curr.daily_quota_limit ||
                prev.weekly_quota_limit !== curr.weekly_quota_limit ||
                prev.monthly_quota_limit !== curr.monthly_quota_limit
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
                return (
                  <div style={{ display: 'flex', gap: '16px', flexWrap: 'wrap' }}>
                    <Form.Item
                      name="quota_limit"
                      label="总额度"
                      initialValue={-1}
                      style={{ flex: 1, minWidth: 120 }}
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
                      style={{ flex: 1, minWidth: 120 }}
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
                      style={{ flex: 1, minWidth: 120 }}
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
                      style={{ flex: 1, minWidth: 120 }}
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
                );
              }}
            </Form.Item>
          )}
          <Form.Item name="base_url" label="端点基础地址 (Base URL)" rules={[{ required: true }]}>
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
              placeholder="可直接选择预设地址或自由输入"
              filterOption={(inputValue, option) =>
                String(option?.label || '').toUpperCase().indexOf(inputValue.toUpperCase()) !== -1 ||
                String(option?.value || '').toUpperCase().indexOf(inputValue.toUpperCase()) !== -1
              }
            />
          </Form.Item>
          <Form.Item
            name="api_key"
            label="请求鉴权密钥 (API Key)"
            extra={editingConfig ? '保持不变直接保存即可，输入新值将覆盖旧密钥' : '可灵 AI 格式：access_key:secret_key；腾讯云 VOD 格式：SecretId:SecretKey:SubAppId；即梦AI 格式：AccessKeyID:SecretAccessKey；其他：sk-xxx'}
          >
            <Input.Password 
              autoComplete="new-password"
              placeholder={editingConfig ? '保持当前密钥或输入新值覆盖' : 'sk-... 或 access_key:secret_key'} 
            />
          </Form.Item>
          <Form.Item name="remark" label="备注说明" extra="在这里记录您的渠道归属、适用场景等信息，方便自己查阅">
            <Input.TextArea rows={2} placeholder="例如：这是供图片生成的官方主通道..." />
          </Form.Item>
          <div style={{ display: 'flex', gap: 24, alignItems: 'flex-start', flexWrap: 'wrap' }}>
            <Form.Item name="sort_order" label="页面排序" extra="数字越大在页面中越靠前" style={{ marginBottom: 0 }}>
              <InputNumber placeholder="0" style={{ width: '120px' }} />
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
          </div>
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
