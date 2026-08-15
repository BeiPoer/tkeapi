/*
 * tokensbyte opensource
 * (c) 2026 tokensbyte.ai
 * @copyright      Copyright netbcloud/wstianxia
 * @license        MIT (https://www.tokensbyte.ai/)
 */

import React, { useEffect, useMemo, useState } from 'react';
import {
  Button,
  Card,
  Checkbox,
  Col,
  Form,
  Input,
  InputNumber,
  Row,
  Space,
  Spin,
  Typography,
  message,
  theme,
} from 'antd';
import { ArrowLeftOutlined, SaveOutlined, SafetyCertificateOutlined } from '@ant-design/icons';
import request from '../../utils/request';
import { fetchActivePlugins } from '../../utils/activePlugins';
import { useNavigate, useParams } from 'react-router-dom';
import useSettingsStore from '../../store/settings';
import { useThemeStore } from '../../store/theme';
import { useTranslation } from 'react-i18next';
import { solidAccent } from '../../theme/tokens';
import type { AdminGroup } from '../../types';
import {
  ADMIN_MENU_PERMISSIONS,
  expandLegacyAdminMenuPermissions,
  flattenAdminMenuPermissions,
  normalizeAdminMenuPermissions,
  type AdminMenuPermChild,
  type AdminMenuPermNode,
} from '../../constants/adminMenuPermissions';

const { Text, Title } = Typography;

const ALL_BASIC_PERMISSION_VALUES = flattenAdminMenuPermissions();

const AdminGroupEdit: React.FC = () => {
  const { token } = theme.useToken();
  const { t } = useTranslation();
  const { themeMode } = useThemeStore();
  const solid = solidAccent(themeMode);
  const muted = themeMode === 'light' ? '#71717a' : '#a1a1aa';

  const { actionId } = useParams<{ actionId: string }>();
  const navigate = useNavigate();
  const { settings } = useSettingsStore();
  const adminPath = settings?.site?.admin_path || 'admin1688';
  const isAdd = actionId === 'new';

  const [form] = Form.useForm();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [activePlugins, setActivePlugins] = useState<any[]>([]);
  const permissionsWatch: string[] = Form.useWatch('permissions', form) || [];
  const pluginPermissionsWatch: string[] = Form.useWatch('plugin_permissions', form) || [];

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      try {
        const pluginsResp = await fetchActivePlugins();
        if (pluginsResp.active_plugins) {
          setActivePlugins(pluginsResp.active_plugins);
        }

        if (isAdd) {
          form.setFieldsValue({
            name: '',
            description: '',
            sort_order: 0,
            permissions: [],
            plugin_permissions: [],
          });
          return;
        }

        const groupsResp = await (request.get('/admin_groups') as any);
        const group: AdminGroup | undefined = (groupsResp.data || []).find(
          (g: AdminGroup) => String(g.id) === actionId,
        );
        if (!group) {
          message.error('未找到对应管理员等级');
          navigate(`/${adminPath}/admin-groups`);
          return;
        }

        const allPerms = group.permissions ? JSON.parse(group.permissions) : [];
        const basicPerms = expandLegacyAdminMenuPermissions(
          allPerms.filter((p: string) => !p.startsWith('plugin:')),
        );
        const pluginPerms = allPerms.filter((p: string) => p.startsWith('plugin:'));
        form.setFieldsValue({
          name: group.name,
          description: group.description,
          sort_order: group.sort_order || 0,
          permissions: basicPerms,
          plugin_permissions: pluginPerms,
        });
      } catch (e) {
        console.error(e);
        message.error('加载失败');
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [actionId, adminPath, form, isAdd, navigate]);

  const setPermissions = (next: string[]) => {
    form.setFieldsValue({ permissions: next });
  };

  const toggleParent = (
    parentValue: string,
    children: { value: string }[] | undefined,
    checked: boolean,
  ) => {
    const childValues = children?.map((c) => c.value) || [];
    const related = [parentValue, ...childValues];
    if (checked) {
      setPermissions(Array.from(new Set([...permissionsWatch, ...related])));
    } else {
      setPermissions(permissionsWatch.filter((p) => !related.includes(p)));
    }
  };

  const toggleChild = (
    parentValue: string,
    childValue: string,
    allChildren: string[],
    checked: boolean,
  ) => {
    let next = checked
      ? Array.from(new Set([...permissionsWatch, childValue, parentValue]))
      : permissionsWatch.filter((p) => p !== childValue);

    const remainingChildren = allChildren.filter((c) => next.includes(c));
    if (remainingChildren.length === 0) {
      next = next.filter((p) => p !== parentValue);
    } else if (!next.includes(parentValue)) {
      next = [...next, parentValue];
    }
    setPermissions(next);
  };

  const basicSelectAllChecked = useMemo(
    () => ALL_BASIC_PERMISSION_VALUES.every((v) => permissionsWatch.includes(v)),
    [permissionsWatch],
  );
  const basicSelectAllIndeterminate = useMemo(
    () =>
      !basicSelectAllChecked &&
      ALL_BASIC_PERMISSION_VALUES.some((v) => permissionsWatch.includes(v)),
    [basicSelectAllChecked, permissionsWatch],
  );

  const selectedMenuCount = useMemo(() => {
    let count = 0;
    for (const group of ADMIN_MENU_PERMISSIONS) {
      if (!group.children?.length) {
        if (permissionsWatch.includes(group.value)) count += 1;
        continue;
      }
      count += group.children.filter((c) => permissionsWatch.includes(c.value)).length;
    }
    return count;
  }, [permissionsWatch]);

  const totalMenuCount = useMemo(() => {
    let count = 0;
    for (const group of ADMIN_MENU_PERMISSIONS) {
      if (!group.children?.length) count += 1;
      else count += group.children.length;
    }
    return count;
  }, []);

  const pluginSelectAllChecked =
    activePlugins.length > 0 &&
    activePlugins.every((p) => pluginPermissionsWatch.includes(`plugin:${p.name}`));
  const pluginSelectAllIndeterminate =
    !pluginSelectAllChecked &&
    activePlugins.some((p) => pluginPermissionsWatch.includes(`plugin:${p.name}`));

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      setSaving(true);
      const basicPerms = normalizeAdminMenuPermissions(values.permissions || []);
      const payload = {
        name: values.name,
        description: values.description,
        sort_order: values.sort_order ?? 0,
        permissions: [...basicPerms, ...(values.plugin_permissions || [])],
      };

      if (isAdd) {
        await request.post('/admin_groups', payload);
        message.success('创建成功');
      } else {
        await request.put(`/admin_groups/${actionId}`, payload);
        message.success('保存成功');
      }
      navigate(`/${adminPath}/admin-groups`);
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  const renderCountBadge = (selected: number, total: number, active: boolean) => (
    <span
      style={{
        fontSize: 11,
        fontWeight: 600,
        lineHeight: '16px',
        padding: '0 5px',
        borderRadius: 4,
        whiteSpace: 'nowrap',
        color: active ? solid.color : muted,
        background: active ? solid.background : 'transparent',
      }}
    >
      {selected}/{total}
    </span>
  );

  const renderMenuGroup = (group: AdminMenuPermNode) => {
    const childValues = group.children?.map((c) => c.value) || [];
    const selectedChildren = childValues.filter((v) => permissionsWatch.includes(v));
    const parentChecked = childValues.length
      ? selectedChildren.length === childValues.length
      : permissionsWatch.includes(group.value);
    const parentIndeterminate =
      childValues.length > 0 &&
      selectedChildren.length > 0 &&
      selectedChildren.length < childValues.length;
    const isActive = parentChecked || parentIndeterminate;
    const childSelectedCount = childValues.length
      ? selectedChildren.length
      : parentChecked
        ? 1
        : 0;
    const childTotalCount = childValues.length || 1;

    return (
      <Col xs={24} sm={12} lg={8} key={group.value}>
        <div style={{ padding: '2px 0 4px' }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              gap: 6,
              marginBottom: group.children?.length ? 2 : 0,
            }}
          >
            <Checkbox
              checked={parentChecked}
              indeterminate={parentIndeterminate}
              onChange={(e) => toggleParent(group.value, group.children, e.target.checked)}
            >
              <Text strong style={{ fontSize: 13 }}>
                {t(group.labelKey, group.label)}
              </Text>
            </Checkbox>
            {renderCountBadge(childSelectedCount, childTotalCount, isActive)}
          </div>

          {group.children && group.children.length > 0 && (
            <div className="admin-perm-check-grid" style={{ paddingLeft: 22 }}>
              {group.children.map((child: AdminMenuPermChild) => (
                <Checkbox
                  key={child.value}
                  checked={permissionsWatch.includes(child.value)}
                  onChange={(e) =>
                    toggleChild(group.value, child.value, childValues, e.target.checked)
                  }
                  className="admin-perm-check-item"
                  style={{ marginInlineEnd: 0 }}
                >
                  <span className="admin-perm-check-label">{t(child.labelKey, child.label)}</span>
                </Checkbox>
              ))}
            </div>
          )}
        </div>
      </Col>
    );
  };

  if (loading) {
    return (
      <div style={{ textAlign: 'center', marginTop: 100 }}>
        <Spin size="large" />
      </div>
    );
  }

  return (
    <div style={{ maxWidth: 1080, margin: '0 auto' }}>
      <Card
        size="small"
        bordered={false}
        styles={{
          body: { padding: '10px 16px 6px' },
          header: { minHeight: 44, padding: '6px 16px' },
        }}
        title={
          <Space size={8}>
            <Button
              size="small"
              icon={<ArrowLeftOutlined />}
              onClick={() => navigate(`/${adminPath}/admin-groups`)}
            />
            <SafetyCertificateOutlined />
            <span style={{ fontWeight: 600, fontSize: 14 }}>
              {isAdd ? '添加管理员等级' : '编辑管理员等级'}
            </span>
          </Space>
        }
        extra={
          <Space size={8}>
            <Button size="small" onClick={() => navigate(`/${adminPath}/admin-groups`)}>
              取消
            </Button>
            <Button
              size="small"
              type="primary"
              icon={<SaveOutlined />}
              loading={saving}
              onClick={handleSave}
            >
              保存
            </Button>
          </Space>
        }
      >
        <Form form={form} layout="vertical" size="small" requiredMark="optional">
          <Form.Item name="permissions" initialValue={[]} hidden>
            <Checkbox.Group options={[]} />
          </Form.Item>
          <Form.Item name="plugin_permissions" initialValue={[]} hidden>
            <Checkbox.Group options={[]} />
          </Form.Item>

          <section style={{ marginBottom: 10 }}>
            <Title level={5} style={{ margin: '0 0 6px', fontSize: 13 }}>
              基本信息
            </Title>
            <Row gutter={[10, 0]}>
              <Col xs={24} sm={10} md={8}>
                <Form.Item
                  name="name"
                  label="分组名称"
                  style={{ marginBottom: 8 }}
                  rules={[{ required: true, message: '请输入分组名称' }]}
                >
                  <Input placeholder="例如：运营管理员" maxLength={64} />
                </Form.Item>
              </Col>
              <Col xs={12} sm={6} md={4}>
                <Form.Item
                  name="sort_order"
                  label="排序"
                  tooltip="数字越大越靠前"
                  initialValue={0}
                  style={{ marginBottom: 8 }}
                >
                  <InputNumber style={{ width: '100%' }} placeholder="0" />
                </Form.Item>
              </Col>
              <Col xs={24} sm={8} md={12}>
                <Form.Item name="description" label="描述" style={{ marginBottom: 8 }}>
                  <Input placeholder="适用场景与权限范围" maxLength={200} />
                </Form.Item>
              </Col>
            </Row>
          </section>

          <section style={{ marginBottom: 10 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                flexWrap: 'wrap',
                gap: 6,
                marginBottom: 4,
              }}
            >
              <Space size={6}>
                <Title level={5} style={{ margin: 0, fontSize: 13 }}>
                  {t('admin_perm.basic_menus')}
                </Title>
                {renderCountBadge(selectedMenuCount, totalMenuCount, selectedMenuCount > 0)}
              </Space>
              <Checkbox
                checked={basicSelectAllChecked}
                indeterminate={basicSelectAllIndeterminate}
                onChange={(e) => {
                  setPermissions(e.target.checked ? [...ALL_BASIC_PERMISSION_VALUES] : []);
                }}
              >
                {t('admin_perm.select_all')}
              </Checkbox>
            </div>
            <Row gutter={[8, 2]}>{ADMIN_MENU_PERMISSIONS.map(renderMenuGroup)}</Row>
          </section>

          {activePlugins.length > 0 && (
            <section style={{ marginBottom: 4 }}>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  flexWrap: 'wrap',
                  gap: 6,
                  marginBottom: 4,
                }}
              >
                <Space size={6}>
                  <Title level={5} style={{ margin: 0, fontSize: 13 }}>
                    {t('admin_perm.plugin_perms')}
                  </Title>
                  {renderCountBadge(
                    pluginPermissionsWatch.length,
                    activePlugins.length,
                    pluginPermissionsWatch.length > 0,
                  )}
                </Space>
                <Checkbox
                  checked={pluginSelectAllChecked}
                  indeterminate={pluginSelectAllIndeterminate}
                  onChange={(e) => {
                    form.setFieldsValue({
                      plugin_permissions: e.target.checked
                        ? activePlugins.map((p) => `plugin:${p.name}`)
                        : [],
                    });
                  }}
                >
                  {t('admin_perm.select_all')}
                </Checkbox>
              </div>
              <div
                className="admin-perm-check-grid"
                style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))' }}
              >
                {activePlugins.map((p) => {
                  const value = `plugin:${p.name}`;
                  return (
                    <Checkbox
                      key={p.name}
                      checked={pluginPermissionsWatch.includes(value)}
                      onChange={(e) => {
                        const set = new Set(pluginPermissionsWatch);
                        if (e.target.checked) set.add(value);
                        else set.delete(value);
                        form.setFieldsValue({ plugin_permissions: Array.from(set) });
                      }}
                      className="admin-perm-check-item"
                      style={{ marginInlineEnd: 0 }}
                    >
                      <span className="admin-perm-check-label">{String(t(`plugin_titles.${p.name}`, { defaultValue: p.title || p.name }))}</span>
                    </Checkbox>
                  );
                })}
              </div>
            </section>
          )}

          <div
            style={{
              marginTop: 10,
              paddingTop: 8,
              display: 'flex',
              justifyContent: 'flex-end',
              gap: 8,
              position: 'sticky',
              bottom: 0,
              background: token.colorBgContainer,
              zIndex: 2,
            }}
          >
            <Button size="small" onClick={() => navigate(`/${adminPath}/admin-groups`)}>
              取消
            </Button>
            <Button
              size="small"
              type="primary"
              icon={<SaveOutlined />}
              loading={saving}
              onClick={handleSave}
            >
              保存
            </Button>
          </div>
        </Form>
      </Card>
    </div>
  );
};

export default AdminGroupEdit;
