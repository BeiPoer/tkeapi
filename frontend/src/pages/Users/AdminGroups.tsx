/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia 
 * @license        MIT (https://www.tokensbyte.ai/)
 */

import React, { useState, useEffect } from 'react';
import { Table, Button, Space, message, Card, Typography, Grid, Tag, Popconfirm } from 'antd';
import MobileCardList, { MobileCard, CardRow, CardActions } from '../../components/MobileCardList';
import { PlusOutlined, EditOutlined, DeleteOutlined, SafetyCertificateOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import request from '../../utils/request';
import { formatApiDateTime } from '../../utils/timedisplay';
import useSettingsStore from '../../store/settings';
import type { AdminGroup } from '../../types';
import {
  ADMIN_MENU_PERMISSIONS,
  expandLegacyAdminMenuPermissions,
  flattenAdminMenuPermissions,
  getAdminMenuPermissionLabel,
} from '../../constants/adminMenuPermissions';

const { Text } = Typography;
const { useBreakpoint } = Grid;

const ALL_BASIC_PERMISSION_VALUES = flattenAdminMenuPermissions();

const AdminGroups: React.FC = () => {
  const [groups, setGroups] = useState<AdminGroup[]>([]);
  const [loading, setLoading] = useState(false);
  const [activePlugins, setActivePlugins] = useState<any[]>([]);
  const screens = useBreakpoint();
  const navigate = useNavigate();
  const { settings } = useSettingsStore();
  const adminPath = settings?.site?.admin_path || 'admin1688';

  const fetchGroups = async () => {
    setLoading(true);
    try {
      const response = await (request.get('/admin_groups') as any);
      setGroups(response.data);
    } catch (error) {
      message.error('获取管理员等级失败');
    } finally {
      setLoading(false);
    }
  };

  const fetchActivePlugins = async () => {
    try {
      const response = await (request.get('/plugins/active') as any);
      if (response.active_plugins) {
        setActivePlugins(response.active_plugins);
      }
    } catch (error) {
      console.error('获取插件失败', error);
    }
  };

  useEffect(() => {
    fetchGroups();
    fetchActivePlugins();
  }, []);

  const handleCreate = () => {
    navigate(`/${adminPath}/admin-groups/new`);
  };

  const handleEdit = (group: AdminGroup) => {
    navigate(`/${adminPath}/admin-groups/${group.id}`);
  };

  const handleDelete = async (id: number) => {
    try {
      await request.delete(`/admin_groups/${id}`);
      message.success('删除成功');
      fetchGroups();
    } catch (error) {
      message.error('删除失败，分组可能正在使用中');
    }
  };

  const renderPermissionsTags = (permissionsStr?: string) => {
    let perms: string[] = [];
    try {
      if (permissionsStr) {
        perms = JSON.parse(permissionsStr);
      }
    } catch {}
    
    const basicPerms = perms.filter((p) => !p.startsWith('plugin:'));
    const expandedBasic = expandLegacyAdminMenuPermissions(basicPerms);
    const hasAllBasic = ALL_BASIC_PERMISSION_VALUES.every((item) => expandedBasic.includes(item));
    const hasAllPlugins =
      activePlugins.length === 0 ||
      activePlugins.every((plugin) => perms.includes(`plugin:${plugin.name}`));

    if (hasAllBasic && hasAllPlugins) {
      return (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
          <Tag color="success" style={{ fontSize: '11px', margin: 0, padding: '0 4px', fontWeight: 'bold' }}>
            全部功能
          </Tag>
        </div>
      );
    }

    const displayKeys: string[] = [];
    for (const group of ADMIN_MENU_PERMISSIONS) {
      if (!group.children?.length) {
        if (expandedBasic.includes(group.value)) displayKeys.push(group.value);
        continue;
      }
      const selectedChildren = group.children.filter((c) => expandedBasic.includes(c.value));
      if (selectedChildren.length === 0) continue;
      if (selectedChildren.length === group.children.length) {
        displayKeys.push(group.value);
      } else {
        selectedChildren.forEach((c) => displayKeys.push(c.value));
      }
    }
    perms.filter((p) => p.startsWith('plugin:')).forEach((p) => displayKeys.push(p));

    const permLabels = displayKeys.map((p) => {
      if (p.startsWith('plugin:')) {
        const pName = p.substring(7);
        const foundPlugin = activePlugins.find((ap) => ap.name === pName);
        return `插件: ${foundPlugin ? (foundPlugin.title || pName) : pName}`;
      }
      return getAdminMenuPermissionLabel(p);
    });

    if (permLabels.length === 0) {
      return <Text type="secondary" style={{ fontSize: '11px' }}>未配置权限</Text>;
    }

    return (
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
        {permLabels.map((label) => (
          <Tag color="blue" style={{ fontSize: '11px', margin: 0, padding: '0 4px' }} key={label}>
            {label}
          </Tag>
        ))}
      </div>
    );
  };

  const columns = [
    { title: '名称', dataIndex: 'name', key: 'name', width: 150 },
    { title: '用户数', dataIndex: 'user_count', key: 'user_count', width: 80, render: (val: number) => <Tag color="blue">{val || 0}</Tag> },
    { 
      title: '权限详细', 
      key: 'permissions_detail', 
      width: 500,
      render: (_: any, record: AdminGroup) => renderPermissionsTags(record.permissions)
    },
    { title: '描述', dataIndex: 'description', key: 'description' },
    { title: '排序', dataIndex: 'sort_order', key: 'sort_order', width: 80 },
    { title: '创建时间', dataIndex: 'created_at', key: 'created_at', render: (text: string) => formatApiDateTime(text) },
    { 
      title: '操作', 
      key: 'action', 
      width: 160,
      render: (_: any, record: AdminGroup) => (
        <Space>
          <Button icon={<EditOutlined />} onClick={() => handleEdit(record)} />
          <Popconfirm title="确认删除该管理员等级？" onConfirm={() => handleDelete(record.id)}>
            <Button danger icon={<DeleteOutlined />} />
          </Popconfirm>
        </Space>
      )
    },
  ];

  return (
    <Card 
      title={
        <Space>
          <SafetyCertificateOutlined />
          <span>管理员权限等级</span>
        </Space>
      }
      extra={
        <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
          添加管理员等级
        </Button>
      }
    >
      {screens.xs ? (
        <MobileCardList
          dataSource={groups}
          loading={loading}
          rowKey="id"
          pagination={false}
          renderCard={(record: any) => (
            <MobileCard
              title={<Text strong>{record.name}</Text>}
              extra={null}
            >
              <CardRow label="权限详细">{renderPermissionsTags(record.permissions)}</CardRow>
              <CardRow label="用户数"><Tag color="blue">{record.user_count || 0}</Tag></CardRow>
              {record.description && <CardRow label="描述"><Text type="secondary" style={{ fontSize: 12 }}>{record.description}</Text></CardRow>}
              <CardRow label="排序"><Text type="secondary" style={{ fontSize: 12 }}>{record.sort_order || 0}</Text></CardRow>
              <CardRow label="创建时间"><Text type="secondary" style={{ fontSize: 12 }}>{formatApiDateTime(record.created_at)}</Text></CardRow>
              <CardActions>
                <Button size="small" icon={<EditOutlined />} onClick={() => handleEdit(record)} />
                <Popconfirm title="确认删除该管理员等级？" onConfirm={() => handleDelete(record.id)}>
                  <Button size="small" danger icon={<DeleteOutlined />} />
                </Popconfirm>
              </CardActions>
            </MobileCard>
          )}
        />
      ) : (
        <Table 
          columns={columns} 
          dataSource={groups} 
          rowKey="id" 
          loading={loading}
        />
      )}
    </Card>
  );
};

export default AdminGroups;
