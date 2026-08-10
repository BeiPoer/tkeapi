import React, { useState, useEffect } from 'react';
import { Switch, Typography, Divider, Button, Form, Input, Select, Row, Col, Card, Space, Tag } from 'antd';
import { SaveOutlined, PlusOutlined, DeleteOutlined, ArrowUpOutlined, ArrowDownOutlined } from '@ant-design/icons';
import { useThemeStore } from '../../../store/theme';

const { Title, Text } = Typography;

const isVideoUrl = (url?: string): boolean => {
  if (!url) return false;
  const cleanUrl = url.split('?')[0].split('#')[0].toLowerCase();
  if (/\.(mp4|webm|ogg|mov|m3u8|m4v|flv)$/i.test(cleanUrl)) return true;
  if (url.startsWith('data:video/')) return true;
  if (url.includes('video/mp4') || url.includes('video/webm')) return true;
  return false;
};

const DEFAULT_HERO_SLIDES = [
  {
    id: 'hero-1',
    category: '图生视频',
    title: '可灵 Kling Video v3 图生视频 [Pro]',
    description: 'Kling 3.0 Pro：顶级图生视频模型，具备电影级视觉效果、流畅的动作生成以及原生音频支持。',
    try_model_id: '',
    docs_url: ''
  },
  {
    id: 'hero-2',
    category: 'SOTA 级视频',
    title: 'Seedance 2.0 旗舰视频生成大模型',
    description: '字节跳动推出的全新 SOTA 级视频模型，支持多参考图、视频与音频同步控制生成。',
    try_model_id: '',
    docs_url: ''
  },
  {
    id: 'hero-3',
    category: '高清图像生成',
    title: 'FLUX 1.1 Pro 高清图像创作引擎',
    description: 'Black Forest Labs 打造的顶级文生图模型，高清晰度细节呈现与极佳的提示词遵循度。',
    try_model_id: '',
    docs_url: ''
  }
];

const DEFAULT_SECTIONS = [
  {
    id: 'sec-seedance',
    title: 'Seedance 2.0 专题',
    description: '字节跳动推出的全新 SOTA 级视频生成模型，即刻体验惊艳的视听合一生成能力。',
    type: 'models',
    items: []
  },
  {
    id: 'sec-grok',
    title: 'Grok Imagine 专题',
    description: '由 xAI 强力驱动的高品质视频、图像与自然语音生成模型系列。',
    type: 'models',
    items: []
  }
];

interface MarketplaceTrendingTabProps {
  config?: any;
  onChange: (config: any) => void;
  onSave: () => void;
  saving?: boolean;
  allModels?: any[];
  allProviders?: any[];
}

const MarketplaceTrendingTab: React.FC<MarketplaceTrendingTabProps> = ({ 
  config, 
  onChange, 
  onSave, 
  saving, 
  allModels = [], 
  allProviders = [] 
}) => {
  const { themeMode } = useThemeStore();
  const _isLight = themeMode === 'light';
  
  const [enabled, setEnabled] = useState<boolean>(config?.enabled || false);
  const [heroSlides, setHeroSlides] = useState<any[]>(
    config?.hero_slides && config.hero_slides.length > 0 ? config.hero_slides : DEFAULT_HERO_SLIDES
  );
  const [sections, setSections] = useState<any[]>(
    config?.sections && config.sections.length > 0 ? config.sections : DEFAULT_SECTIONS
  );

  useEffect(() => {
    if (config) {
      if (config.enabled !== undefined) setEnabled(config.enabled);
      if (config.hero_slides && config.hero_slides.length > 0) {
        setHeroSlides(config.hero_slides);
      } else if (config.hero) {
        // Fallback for single hero config migrating to slides
        setHeroSlides([{ id: 'hero-1', ...config.hero }, ...DEFAULT_HERO_SLIDES.slice(1)]);
      }
      if (config.sections && config.sections.length > 0) setSections(config.sections);
    }
  }, [config]);

  const triggerChange = (newEnabled: boolean, newHeroSlides: any[], newSections: any[]) => {
    onChange({
      ...config,
      enabled: newEnabled,
      hero_slides: newHeroSlides,
      hero: newHeroSlides.length > 0 ? newHeroSlides[0] : {},
      sections: newSections
    });
  };
  const handleEnabledChange = (val: boolean) => {
    setEnabled(val);
    triggerChange(val, heroSlides, sections);
  };

  // Hero Slide operations
  const handleAddHeroSlide = () => {
    const newSlide = {
      id: `hero-${Date.now()}`,
      category: '推荐模型',
      title: '新 Banner 推荐标题',
      description: '请输入推荐模型的详细说明文字...',
      bg_image: '',
      try_model_id: '',
      docs_url: ''
    };
    const updated = [...heroSlides, newSlide];
    setHeroSlides(updated);
    triggerChange(enabled, updated, sections);
  };

  const handleRemoveHeroSlide = (index: number) => {
    const updated = heroSlides.filter((_, i) => i !== index);
    setHeroSlides(updated);
    triggerChange(enabled, updated, sections);
  };

  const handleHeroSlideChange = (index: number, field: string, val: any) => {
    const updated = [...heroSlides];
    if (field === 'try_model_id' && val) {
      const targetId = typeof val === 'string' && val.startsWith('orig:') ? val.replace('orig:', '') : val;
      const selectedModel = allModels?.find(m => 
        m.mid === val || 
        m.id?.toString() === val || 
        m.original_id === val || 
        m.original_id === targetId ||
        m.model_id === targetId ||
        m.name === val ||
        m.name === targetId
      );
      if (selectedModel) {
        const autoCategory = selectedModel.type_name || selectedModel.category || '';
        const autoTitle = selectedModel.name || selectedModel.original_id || targetId || '';
        const autoDesc = selectedModel.description || selectedModel.model_description || '';
        updated[index] = { 
          ...updated[index], 
          try_model_id: val,
          category: autoCategory || updated[index].category,
          title: (!updated[index].title || updated[index].title === '新 Banner 推荐标题') ? autoTitle : updated[index].title,
          description: (!updated[index].description || updated[index].description === '请输入推荐模型的详细说明文字...') ? autoDesc : updated[index].description
        };
      } else {
        updated[index] = { ...updated[index], [field]: val };
      }
    } else {
      updated[index] = { ...updated[index], [field]: val };
    }
    setHeroSlides(updated);
    triggerChange(enabled, updated, sections);
  };

  const handleMoveHeroSlide = (index: number, direction: 'up' | 'down') => {
    const targetIndex = direction === 'up' ? index - 1 : index + 1;
    if (targetIndex < 0 || targetIndex >= heroSlides.length) return;
    const updated = [...heroSlides];
    const temp = updated[index];
    updated[index] = updated[targetIndex];
    updated[targetIndex] = temp;
    setHeroSlides(updated);
    triggerChange(enabled, updated, sections);
  };

  // Section operations
  const handleAddSection = () => {
    const newSec = {
      id: `sec-${Date.now()}`,
      title: '新热门专题',
      description: '请输入专题描述信息...',
      type: 'models',
      items: []
    };
    const updatedSections = [...sections, newSec];
    setSections(updatedSections);
    triggerChange(enabled, heroSlides, updatedSections);
  };

  const handleRemoveSection = (index: number) => {
    const updatedSections = sections.filter((_, i) => i !== index);
    setSections(updatedSections);
    triggerChange(enabled, heroSlides, updatedSections);
  };

  const handleSectionChange = (index: number, field: string, val: any) => {
    const updatedSections = [...sections];
    updatedSections[index] = { ...updatedSections[index], [field]: val };
    if (field === 'type') {
      updatedSections[index].items = [];
    }
    setSections(updatedSections);
    triggerChange(enabled, heroSlides, updatedSections);
  };

  const handleMoveSection = (index: number, direction: 'up' | 'down') => {
    const targetIndex = direction === 'up' ? index - 1 : index + 1;
    if (targetIndex < 0 || targetIndex >= sections.length) return;
    const updatedSections = [...sections];
    const temp = updatedSections[index];
    updatedSections[index] = updatedSections[targetIndex];
    updatedSections[targetIndex] = temp;
    setSections(updatedSections);
    triggerChange(enabled, heroSlides, updatedSections);
  };

  // Extract all unique model categories/groups from allModels
  const extractedGroups = Array.from(new Set(
    (allModels || [])
      .map((m: any) => m.type_name || m.category)
      .filter(Boolean)
  ));
  
  // Default common categories if list is short
  const defaultCategories = ['图生视频', '文生视频', '视频生视频', '文生图', '图生图', '文本生成', '对话模型', '文本转语音', '代码生成', '多模态'];
  const allCategoryNames = Array.from(new Set([...extractedGroups, ...defaultCategories]));

  const groupOptions = allCategoryNames.map((cat: string) => ({
    label: `📦 模型组/类型: ${cat}`,
    value: `group:${cat}`
  }));

  const rawModelOptions = (allModels || []).map((m: any) => ({
    label: `${m.name || m.original_id || m.model_id || '未命名模型'} (${m.provider_name || '公共'})`,
    value: m.mid || m.original_id || m.model_id || m.id?.toString() || m.name
  }));

  const modelOptions = rawModelOptions;

  // Group models by original_id to build multi-price model options
  const origIdMap = new Map<string, any[]>();
  (allModels || []).forEach((m: any) => {
    const origId = m.original_id || m.model_id;
    if (origId) {
      if (!origIdMap.has(origId)) {
        origIdMap.set(origId, []);
      }
      origIdMap.get(origId)!.push(m);
    }
  });

  const originalIdOptions = Array.from(origIdMap.entries())
    .map(([origId, items]) => {
      const displayName = items[0]?.name || origId;
      const count = items.length;
      return {
        label: `🏷️ 原始ID模型组: ${displayName} (${origId}) [${count}个价格渠道]`,
        value: `orig:${origId}`
      };
    });

  const combinedModelAndGroupOptions = [
    {
      label: '🏷️ 同原始ID模型组 (支持多渠道价格展示)',
      options: originalIdOptions
    },
    {
      label: '📦 能力分类模型组 (选中可包含该类型全部模型)',
      options: groupOptions
    },
    {
      label: '🤖 独立站点模型渠道',
      options: rawModelOptions
    }
  ];

  const providerOptions = (allProviders || []).map((p: any) => ({
    label: p.name,
    value: p.id?.toString() || p.name
  }));

  return (
    <div style={{ maxWidth: 840, paddingBottom: 40 }}>
      {/* Switch Header */}
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: 20 }}>
        <Text strong style={{ width: 140, fontSize: 15 }}>开启热门推荐</Text>
        <Switch checked={enabled} onChange={handleEnabledChange} />
        <Text type="secondary" style={{ marginLeft: 12 }}>开启后将在用户端模型广场左侧边栏新增“热门推荐”入口</Text>
      </div>

      <Divider />

      {/* Hero Header Section Config */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <div>
          <Title level={5} style={{ margin: 0 }}>顶部 Banner 轮播推荐配置</Title>
          <Text type="secondary" style={{ fontSize: 13 }}>可自定义编辑、增加或轮播切换前台顶部的推荐模型数据</Text>
        </div>
        <Button type="primary" size="small" icon={<PlusOutlined />} onClick={handleAddHeroSlide}>
          新增 Banner 推荐
        </Button>
      </div>

      {heroSlides.map((slide, idx) => (
        <Card
          key={slide.id || idx}
          size="small"
          style={{ 
            marginBottom: 16, 
            background: _isLight ? '#ffffff' : '#141416', 
            borderColor: _isLight ? '#e5e7eb' : '#27272a' 
          }}
          title={
            <Space>
              <Text strong style={{ fontSize: 14 }}>{`Banner 项 ${idx + 1}: ${slide.title || '未命名 Banner'}`}</Text>
            </Space>
          }
          extra={
            <Space>
              <Button 
                type="text" 
                size="small" 
                disabled={idx === 0} 
                icon={<ArrowUpOutlined />} 
                onClick={() => handleMoveHeroSlide(idx, 'up')} 
              />
              <Button 
                type="text" 
                size="small" 
                disabled={idx === heroSlides.length - 1} 
                icon={<ArrowDownOutlined />} 
                onClick={() => handleMoveHeroSlide(idx, 'down')} 
              />
              <Button 
                type="text" 
                danger 
                size="small" 
                icon={<DeleteOutlined />} 
                disabled={heroSlides.length <= 1}
                onClick={() => handleRemoveHeroSlide(idx)} 
              >
                删除
              </Button>
            </Space>
          }
        >
          <Row gutter={[16, 16]}>
            <Col span={24}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>Banner 标题</Text>
              <Input 
                value={slide.title} 
                onChange={e => handleHeroSlideChange(idx, 'title', e.target.value)} 
                placeholder="例如：可灵 Kling Video v3 图生视频 [Pro]" 
              />
            </Col>
            <Col span={24}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>Banner 描述</Text>
              <Input.TextArea 
                value={slide.description} 
                onChange={e => handleHeroSlideChange(idx, 'description', e.target.value)} 
                rows={2} 
                placeholder="例如：Kling 3.0 Pro：顶级图生视频模型..." 
              />
            </Col>
            <Col span={24}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>幻灯片背景图片/视频 URL (可选)</Text>
              <Input 
                value={slide.bg_image} 
                onChange={e => handleHeroSlideChange(idx, 'bg_image', e.target.value)} 
                placeholder="请输入背景图片或视频 URL（支持 .mp4, .webm, .m3u8, .mov 或网络/本地媒体链接）" 
                allowClear
              />
              <Text type="secondary" style={{ fontSize: 11, display: 'block', marginTop: 4 }}>
                支持图片 (JPG/PNG/WebP/GIF) 或视频 (MP4/WebM/M3U8) 链接。填入视频链接时，前台将自动在 Banner 背景静音无缝循环播放。
              </Text>
              {slide.bg_image && (
                <div style={{ marginTop: 8, display: 'flex', alignItems: 'center', gap: 10 }}>
                  <Tag color={isVideoUrl(slide.bg_image) ? 'blue' : 'green'}>
                    {isVideoUrl(slide.bg_image) ? '视频背景' : '图片背景'}
                  </Tag>
                  {isVideoUrl(slide.bg_image) ? (
                    <video
                      src={slide.bg_image}
                      muted
                      style={{ width: 100, height: 56, objectFit: 'cover', borderRadius: 4, border: `1px solid ${_isLight ? '#d9d9d9' : '#333'}` }}
                    />
                  ) : (
                    <img
                      src={slide.bg_image}
                      alt="背景预览"
                      style={{ width: 100, height: 56, objectFit: 'cover', borderRadius: 4, border: `1px solid ${_isLight ? '#d9d9d9' : '#333'}` }}
                      onError={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
                    />
                  )}
                </div>
              )}
            </Col>
            <Col span={12}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>"立即体验" 关联模型 / 原始ID模型组</Text>
              <Select
                showSearch
                allowClear
                optionFilterProp="label"
                style={{ width: '100%' }}
                value={slide.try_model_id}
                onChange={val => handleHeroSlideChange(idx, 'try_model_id', val)}
                options={combinedModelAndGroupOptions}
                placeholder="选择点击体验时关联的模型或同原始ID模型组"
              />
            </Col>
            <Col span={12}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>"查看文档" 链接 (可选)</Text>
              <Input 
                value={slide.docs_url} 
                onChange={e => handleHeroSlideChange(idx, 'docs_url', e.target.value)} 
                placeholder="https://..." 
              />
            </Col>
          </Row>
        </Card>
      ))}

      <Divider />

      {/* Topics / Sections Manager */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <div>
          <Title level={5} style={{ margin: 0 }}>热门推荐专题 (Sections) 管理</Title>
          <Text type="secondary" style={{ fontSize: 13 }}>可自定义新增或删除专题，并将站点中的大模型或服务商添加到对应专题中展示</Text>
        </div>
        <Button type="primary" size="small" icon={<PlusOutlined />} onClick={handleAddSection}>
          新增专题
        </Button>
      </div>

      {sections.map((sec, idx) => (
        <Card
          key={sec.id || idx}
          size="small"
          style={{ 
            marginBottom: 16, 
            background: _isLight ? '#ffffff' : '#141416', 
            borderColor: _isLight ? '#e5e7eb' : '#27272a' 
          }}
          title={
            <Space>
              <Text strong style={{ fontSize: 14 }}>{`专题 ${idx + 1}: ${sec.title || '未命名专题'}`}</Text>
            </Space>
          }
          extra={
            <Space>
              <Button 
                type="text" 
                size="small" 
                disabled={idx === 0} 
                icon={<ArrowUpOutlined />} 
                onClick={() => handleMoveSection(idx, 'up')} 
              />
              <Button 
                type="text" 
                size="small" 
                disabled={idx === sections.length - 1} 
                icon={<ArrowDownOutlined />} 
                onClick={() => handleMoveSection(idx, 'down')} 
              />
              <Button 
                type="text" 
                danger 
                size="small" 
                icon={<DeleteOutlined />} 
                onClick={() => handleRemoveSection(idx)} 
              >
                删除专题
              </Button>
            </Space>
          }
        >
          <Row gutter={[16, 16]}>
            <Col span={14}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>专题标题</Text>
              <Input 
                value={sec.title} 
                onChange={e => handleSectionChange(idx, 'title', e.target.value)} 
                placeholder="例如：Seedance 2.0 专题" 
              />
            </Col>
            <Col span={10}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>专题类型</Text>
              <Select
                style={{ width: '100%' }}
                value={sec.type}
                onChange={val => handleSectionChange(idx, 'type', val)}
                options={[
                  { label: '模型列表 / 模型组 (Models)', value: 'models' },
                  { label: '仅模型组/类型 (Model Groups)', value: 'groups' },
                  { label: '模型厂家 (Providers)', value: 'providers' }
                ]}
              />
            </Col>
            <Col span={24}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>专题副标题/描述</Text>
              <Input 
                value={sec.description} 
                onChange={e => handleSectionChange(idx, 'description', e.target.value)} 
                placeholder="专题补充描述说明" 
              />
            </Col>
            <Col span={24}>
              <Text style={{ display: 'block', marginBottom: 4, fontSize: 12 }}>
                {`关联数据内容 (${sec.type === 'models' ? '选择站点模型或模型组' : (sec.type === 'groups' ? '选择模型组' : '选择站点模型厂家')})`}
              </Text>
              <Select
                mode="multiple"
                showSearch
                allowClear
                optionFilterProp="label"
                style={{ width: '100%' }}
                value={sec.items}
                onChange={val => handleSectionChange(idx, 'items', val)}
                options={(sec.type === 'models' ? combinedModelAndGroupOptions : (sec.type === 'groups' ? groupOptions : providerOptions)) as any}
                placeholder={
                  sec.type === 'models' 
                    ? '请选择要展示的站点模型或模型组（按模型组选择可包含该类型全部模型）' 
                    : (sec.type === 'groups' ? '请选择该专题要展示的模型组/分类' : '请选择该专题要展示的服务商')
                }
              />
            </Col>
          </Row>
        </Card>
      ))}

      {sections.length === 0 && (
        <div style={{ textAlign: 'center', padding: '32px 0', color: _isLight ? '#999' : '#555', border: '1px dashed #d9d9d9', borderRadius: 8, marginBottom: 24 }}>
          暂无推荐专题，点击右上角“新增专题”进行添加
        </div>
      )}

      <Divider />

      {/* Save Button */}
      <Button 
        type="primary" 
        size="large"
        icon={<SaveOutlined />} 
        onClick={onSave} 
        loading={saving}
        style={{ padding: '0 32px' }}
      >
        保存热门推荐配置
      </Button>
    </div>
  );
};

export default MarketplaceTrendingTab;
