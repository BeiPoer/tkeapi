# UPDATE

## 2026-08-15 — 开源版插件配置页不再因 HaLogs 静态导入崩溃
- `PluginConfig` 改为经 `plugins-registry` 的 `import.meta.glob` 加载高可用日志页，剥离目录时 Vite 不再在 import-analysis 阶段失败
- 开源构建保留 `HighAvailability/`（`high_availability_channel` 本就是开源插件）

## 2026-08-15 — 支付渠道安装默认中英文副标题
- 系统全新安装后，在线支付设置未自定义时使用当前站点已配置的渠道副标题：支付宝/微信「快捷 / code pay」，通联「微信/支付宝/信用卡 / wechat/alipay」，Stripe「银行卡/支付宝 / Cards/Alipay」，BonusPay 与 HyperBC「Web3 / Web3」

## 2026-08-15 — 清空数据库后进入与全新安装相同的初始化流程
- 管理后台「初始化并清空数据库」成功后立即清除登录态，整页进入管理员初始化；清空后不再杀掉后端进程（只清内存缓存），避免本地卡在「服务重启中」
- 探测接口带超时；若仍短暂连不上，页面提示确认后端已启动后刷新

## 2026-08-15 — 兑换管理列表支持按活动名称筛选查找
- 管理后台「兑换管理」列表页新增活动名称搜索框，支持输入关键词即时与回车模糊匹配筛选查找对应活动分组；
- 后端接口 `/redemptions/groups` 增加 `name` 参数模糊过滤与分页计数支持，保持开源与商业版同步。

## 2026-08-15 — 管理后台数据库设置改为只读展示
- 「数据库设置」只显示当前 PostgreSQL 连接（来自 `DATABASE_URL` / `data/.database_url`），字段不可编辑、不能切换连接
- 保留「测试连接」（测当前库）和「初始化并清空数据库」：须输入「确认清空当前数据」，再倒计时 10 秒，期间可取消；到期后清空 public schema 并重建表结构，服务重启后需重新初始化管理员
- 附加信息展示库运行时长、大小、连接数、缓存命中率、事务与站点进程状态；只读目录/共享内存快照，打开或刷新时查一次，不轮询、不扫业务表
- 「异常计费订正」仍在「数据清理」

## 2026-08-15 — 在线支付设置列表：支付通道中英文配置分组与换行直接编辑
- 支付渠道列表将「中文配置」与「英文配置」明确分组展示为独立列；中文名称换行显示副标题，英文名称换行显示英文副标题，均支持在列表中直接编辑与失焦/回车即时保存
- 支付渠道配置详情页同步优化为中英文配置卡片栅格分组布局

## 2026-08-15 — Create Token 页面标题多语言
- 创建/编辑令牌表单的字段标题、说明、占位与确认弹窗全部走 i18n；缺省词条已写入中/英/繁/日/韩/越语言包

## 2026-08-15 — 管理后台菜单英文补全
- 侧栏、系统关于开源/商业版、站点插件列表与管理员等级「可见菜单」权限树全部走 i18n；英文词条按中文含义补全（如存储设置、模型渠道分组、系统资金明细）

## 2026-08-15 — 首次安装站点默认时区取浏览器时区
- 系统尚未初始化管理员时，在管理后台设置首个超级管理员密码并登录，站点默认时区写入当前浏览器 IANA 时区（非法值仍保留服务器默认）
- 系统运行时区仍固定 UTC（进程 TZ、数据库 `TIME ZONE`、管理端灰框硬编码 UTC）；首次安装只改站点业务时区，不可改运行时区

## 2026-08-15 — 站点语言增加繁體中文
- 管理后台「基础设置 → 站点信息 → 站点语言管理」在简体中文后增加繁體中文（`zh-TW`），旗帜为香港
- 控制台、登录页、门户与文档均提供繁體中文界面；缺省词条回退简体中文

## 2026-08-15 — 支付渠道配置返回停留在渠道 Tab
- 在线支付设置里从渠道配置点「返回」，回到支付渠道列表，不再落到货币设置

## 2026-08-15 — 钱包充值页与支付通道中英文名称
- 用户端充值弹窗剩余中文文案全部走 i18n，站点语言为英文时不再露出「点击快捷选择」「已选 支付宝」等中文
- 在线支付设置可为每个支付通道分别填写中文/英文显示名称与副标题；中文站点用中文名，其他语言用英文名（未填则用系统默认）

## 2026-08-14 — 阿里百炼 multimodal 生图响应识别
- 官方张数认 `usage.output_image_count`；数组计数对齐 `output.choices[].message.content[].image`，避免成功体被判「无有效图片」
- OpenAI 转换沿用已有 URL 提取与 usage 挂载

## 2026-08-14 — logs 慢查询：task_id 回表与仪表盘今日聚合去重
- 异步轮询按 `task_id` 先取最新 `id` 再回表，避免大字段 Bitmap Heap 拖垮连接池；补 `(task_id, id DESC)` 复合索引
- 仪表盘今日 logs 只聚合一次（合计/卡片/模型明细共用），COUNT/SUM/DISTINCT 合并；最近活动不再 `SELECT l.*`
- pending 轮询补 `is_completed=0 AND status_code=200` 部分索引；`SUM(prompt_tokens+completion_tokens)` 改为分列 SUM 以走 covering

## 2026-08-14 — 用户等级与管理员等级增加搜索
- 用户等级管理页与管理员等级管理页增加搜索框，支持按等级名称、等级 ID 模糊与精准搜索

## 2026-08-14 — 上游渠道配置支持 NewAPI 分组倍率同步
- 编辑上游渠道配置时，密钥下方可选手动填写「上游系统」（兼容 / 官方 / newapi / akeapi / 火山引擎 / 阿里云）
- 选择 newapi 后可用当前地址与密钥拉取分组倍率，选中某一分组写入本渠道倍率；可选每 N 分钟自动同步，以及同步时叠加的增量
- 已配置同步的渠道，在列表「配置」与「状态」之间展示上游系统、分组倍率、同步间隔与增量

## 2026-08-14 — 模型列表默认按名称排序，新模型暂时置顶
- 默认按模型名称首字母（中文拼音 / 英文）排列
- 刚添加的模型暂时排在第一位，刷新或改排序后恢复名称序

## 2026-08-14 — 模型列表：系统预设筛选与安装种子
- 管理后台「模型列表管理」标题旁增加模型分类筛选：全部模型 / 系统预设 / 自定义
- 模型表新增 `is_system`；系统预设不可删除（可禁用）；安装时写入 Seedance / Kling 等官方计费与转发规则已绑定的预设模型
- 火山画质增强等插件预置模型一并标为系统预设

## 2026-08-14 — ComfyUI 渠道多节点与调用规则
- 渠道「选择上游渠道」支持多选 ComfyUI 服务节点，并下拉选择调用规则（权重优先 / 随机 / 顺序 / 空闲优先）
- 插件服务节点增加优先级、权重、排序；新增「调用规则」页，可开关上述四种系统规则
- 提交时按所选规则从已选节点中挑一台；旧渠道仅绑单个 `comfyui_server_id` 仍可用

## 2026-08-14 — 插件配置页去掉标题与 Tab 间空白
- 管理端插件配置页头取消多余下边距与分隔线，标题紧贴 Tab，少占一截空白

## 2026-08-14 — ComfyUI 节点硬件与队列
- 管理端「服务地址」列表按节点代理 ComfyUI `GET /system_stats`、`GET /queue`，展示 GPU/显存/内存与运行中、排队任务
- 队列详情只回传 prompt_id 等摘要，不下发整份 prompt 图

## 2026-08-14 — ComfyUI 工作流与服务节点多对多
- 导入工作流：「服务节点」可选、可多选，不必先绑节点也能保存
- 新增/编辑服务节点时可勾选支持的工作流；两边写同一张绑定表

## 2026-08-13 — ComfyUI 服务节点即渠道
- 渠道分组「开启 ComfyUI」改为选择服务节点（不再选工作流）；节点的地址实时覆盖渠道基址
- 工作流仍由模型转发规则或该节点已应用的工作流解析；旧渠道仅绑 `comfyui_workflow_id` 仍可用

## 2026-08-13 — 合并 origin/chenzs
- 并入 HA 日志日期筛选/尝试次数排序、纯成功不入库、402 欠费识别；DashScope 异步头仅 POST；日统计 SQL；火山素材保留天数默认 30
- 保留本分支 ComfyUI 接入（任务表关联 logs）

## 2026-08-13 — ComfyUI 接入插件
- 系统增强插件 `comfyui_bridge`：管理 ComfyUI 地址与工作流（画布 JSON 导入后「应用」编译），插件内生成转发规则（不 SQL 预设）
- 渠道分组可「开启 ComfyUI 渠道」并选择已应用工作流（与火山画质增强同路径）；选渠后走系统 relay，渠道工作流优先于模型转发规则
- 识别 Save 画布 / Export API / `{workflow}` 包装；多分组按输出节点裁剪（如 FL2VA 与 Ref2VA 不同时跑）；可视化改提示词、尺寸并导入 JSON
- Relay 仅在 `target_type=comfyui` 时走 submit/poll；轮询日志 `url` 为真实 `GET {base}/history/{prompt_id}`
- 成片：插件 TOS → 基础设置 TOS → 本地 `/assets/comfyui/`（公开基址落盘时 `infer_base_url`，不入库）
- 任务日志：JOIN `logs` 展示时间/状态/错误/载荷；`comfyui_jobs.log_id` = `logs.id`，仅存 prompt / 工作流 / 服务 / 成片 URL
- 轮询失败只落 `execution_error.exception_message`（节点 id/类型），不把 current_inputs 张量写入 logs
- 下发 ComfyUI 的 `/prompt` JSON 写入 `logs.upstream_req_content`，任务日志可对照用户入参与实际工作流

## 2026-08-13 — pending 结算 CAS 未命中不再二次补偿
- `record_and_bill_inner`：pending 更新 0 行用 `pending_cas_miss` 区分；已关单时跳过补偿退款，避免无效关单调用
- `run_daily_stats_loop`：去掉对恒为 Ok 的外层错误分支（失败已在 `sync_daily_stats` 内记录）

## 2026-08-13 — 删除 forward_rules 残留单测
- 去掉 `forward_rules.rs` 内 `#[cfg(test)]`（keep-system-lean）；业务隐藏规则逻辑不变；保留产品页「通道测试」

## 2026-08-13 — 日清理多批死循环修复与 pending 退款路径合并
- 素材/上游缓存：本地删除失败或 0 行时停止续批，避免日任务空转死循环
- `close_pending_and_refund` / `refund_pending` 合并为 `settle_pending_prepay`
- 每日 UTC 03:00 等待抽 `duration_until_next_utc_hms`；数据清理文案去掉误写的 API 素材
- DailyStats：0 点窗内未成功则 5 分钟重试，成功后睡到次日零点；HA 无预扣跳过无用行锁
- 小提炼：`local_ymd_num` 复用；pending 关单日志分支收拢（不改行为）

## 2026-08-13 — 按天清理任务降频并合并空转 cron
- 火山素材 + TOS 过期并入每日 UTC 03:00 维护（与日志大字段清理同 tick）；日任务内多批直到清空或限流
- `DailyStatsSync` 改为睡到站点时区次日 00:00，去掉每 5 分钟空唤醒
- Playground / Playground2026 节点恢复合并为同一 5 分钟 tick
- 顺带修正每日 03:00 等待为「下一次」而非总是「明天」

## 2026-08-13 — pending 关单退款同行锁读金额，结算 CAS 防双动账
- `close_pending_and_refund` / `refund_pending`：`FOR UPDATE` 读行内 `cost`/`pre_deduct_gift` 后退款，去掉事务外 SELECT 金额
- 孤儿/启动恢复只查 `id`；去掉启动恢复对「冻结」的 LIKE 兜底（冻结本为 `status_code=200`）
- pending 结算 `UPDATE` 增加 `status_code=0` CAS，关单后退款事务整笔回滚，避免覆盖已退日志再扣费

## 2026-08-12 — 异步成功结算与 response_content 同 CAS，防 pending 覆盖
- `execute_settlement_tx` 结算时同写终态 `response_content`；GET/后台去掉结算后二次 UPDATE
- 中间态/失败落库经 `persist_open_poll_response`（及级联 S2 失败）仅 `is_completed=0` 可写，避免已扣费日志仍显示 running

## 2026-08-12 — 火山素材清理配置并入数据清理
- 保留天数改为站点设置「数据清理 → 火山素材保留天数」（`storage.volc_asset_retention_days`，默认 7，0=关闭）
- 去掉各插件存储页分散配置；定时任务统一入口 `cleanup_expired_volc_assets`
- 每 10 分钟每类最多 20 条、条间 500ms；遇 429 本批本地不删，且只跳过同一凭证后续类（国际版/上游仍继续）
- 自动清理仅 `relay_convert` + 上游转素材缓存；**不删** `api_proxy`（用户 API 素材）及用户上传/预设

## 2026-08-12 — 火山素材清理遇限流停本轮并节流
- DeleteAsset 条间间隔 500ms；识别 429 / AccountFlowLimitExceeded 等限流后立即停止剩余
- 定时清理改为先云端再本地：限流时本批本地不删，留待下次；单批上限 20
- 管理端异步批删同样节流+遇限流中止剩余云端调用

## 2026-08-12 — 火山素材库默认 7 天自动清理
- `asset_manager` / `asset_manager_intl`：定时清理过期 `relay_convert`、`api_proxy`（本地 + TOS 尽力 + 方舟 DeleteAsset）
- `upstream_asset_relay`：定时清理过期 `upstream_relay_convert` 缓存（本地 + 上游 DeleteAsset）
- 后台每 10 分钟跑 `VolcAssetRetentionCleanup`；同 asset_id 仍有引用时跳过云端删

## 2026-08-12 — 腾讯云视频：单图未标 role 推断为首帧
- `tc_collect_video_src` 复用 `infer_image_default_roles`（与方舟等一致）：1→FirstFrame，2→首尾，3+→Reference；显式 role/type 仍优先

## 2026-08-12 — 日志列表不查 plugin_tag
- `/logs` 列表 SELECT 去掉 `plugin_tag`；展开 `/logs/{id}/detail` 按需返回（用户端仅投影 `client_ct`）
- 仪表盘最近活动清空 `plugin_tag`；列表脱敏不再投影该字段
- 前端使用日志 / 任务日志共用 `parsePluginTagMeta`，展开读详情缓存

## 2026-08-12 — 用户列表模型折扣悬停恢复网格 Tooltip
- 恢复早期友好布局：悬停直接显示完整模型名 + 倍率；`placement=right`、放宽 maxWidth，一次看清无需再悬停名称

## 2026-08-12 — 日志详情超管可见级联 S1 上游任务 ID
- 使用日志 / 任务日志下拉详情：`plugin_tag.cascade.s1_task_id`；仅 `admin && !admin_group_id`

## 2026-08-12 — logs 短路：先 status 再级联，处理中用 response
- `try_client_poll_from_logs`：无 status 先打上游；未完成级联 ack 取 `response.stage1` 否则整包；`is_cascade` 紧贴使用处
- 规范：改根因勿堆兜底（全项目）

## 2026-08-12 — 后台轮询周期可配
- `RelaySettings.poll_tick_secs`（`prepared` 内 0→30 再钳 5–300）写入「模型调用设置」；TaskPoller 读已缓存字段，保存写穿

## 2026-08-12 — merge origin/cgdev0808 → chenzs
- 合并门户/日志/用户列表等 cgdev0808 改动与本分支轮询级联精简；冲突处保留双方功能（RelaySettings、支付 Divider TS 修复等）

## 2026-08-12 — 轮询类别直接用 logs.action_type
- 去掉 `task::infer_category`（path 兜底 + models 表二次查询）；手动/后台轮询与 logs 短路一律用 POST 已写入的 `action_type`
- `TASK_RELAY_LOG_COLS` 去掉无用的 `endpoint`；`action_type` SELECT 为 `category`，结构体/调用统一用 `category`
- `task_status` / `sync_single_task` 共用同一套 SELECT 与整行 `log`；去掉 `PollLogView` / `db_log_id` / `model_name` 等中间重命名；缺行错误文案收短为「任务不存在」「渠道不存在」「缺少 model」
- 轮询/级联：日志与对外文案收短；抽出 `category_hint`；注释去冗（行为不变）
- S2 无任务 id 兜底 settle：`cascade_poll_target` 返回 `(文案, status)`，优先从 stage2 体推断，无法识别才 500

## 2026-08-11 — 级联 S2 提交入参收成 CascadeS2SubmitCtx
- `cascade_stage2_submit` / `try_cascade_stage2_submit` 用上下文结构替代 10+ 散参；行为不变

## 2026-08-11 — 级联未完成/失败禁止泄漏 S1 底座成片
- `cascade_sanitize_for_user`：已完成失败或 S2 无产物 URL 时返回 failed，不再 `cascade_s1_with_s2_url` 回退 S1
- `try_client_poll_from_logs`：关手动上游时未完成级联只回「处理中」；完成成功须 S2 有产物 URL 才 format 成功体
- 无产物硬保证收进 `cascade_s2_client_processing`（live/logs 共用）；task 去掉重复 find_urls 兜底；ack 只取 post_response
- 精简：内联 `cascade_clamp_base_resolution` / `apply_cascade_res_mul_to_stage1`；对外 fps 默认与提交一致为 24

## 2026-08-11 — 手动/后台终态段就地精简
- GET ~711–951 与 `sync_single_task` 终态：`mut` 压扁链式 `let`、结算用 `match`、对外体用 `(stage, status)`；行为不变（腾讯类别/align 时机/callback 路径保持差异）
- 仅抽出两处共用的 `persist_cascade_s2_fail`，不再堆单点小函数

## 2026-08-11 — 级联 stage2 无 id：去掉重复结案写库
- 正常 S2 提交失败已在 `try_cascade_stage2_submit` 结案；轮询再遇无 id 仅 `settle_failure` CAS 补洞，不再二次改 response_content

## 2026-08-11 — task/cascade 解耦精简（结算环 + 共用轮询）
- cascade 不再调用 `settle_failure` / `execute_refund_tx`；S2 提交失败与无 task_id 由 task 统一结案
- GET / 后台共用 `run_upstream_poll`、`try_cascade_stage2_submit`；`CascadeMk` 收为 cascade 内部
- 去掉空 `task_id` 等不变量死兜底；MediaKit 仍复用 `task::poll_task_result`

## 2026-08-11 — 级联轮询目标解析解耦精简
- `cascade_poll_target` + `cascade_settle_s2_no_task_id` 收口到 `cascade.rs`，手动 GET / 后台轮询共用
- 去掉 `cascade_resolve_s2_poll` / `cascade_s1_upstream_task_id` / `pick_poll_target` 等碎片 API
- 上游轮询路径 `db_log_id` 定为 `i64`（无 logs 行时选渠已失败），去掉多余 `Option` 判断

## 2026-08-11 — 手动轮询关上游：缓存无 status 时兜底打上游
- `try_client_poll_from_logs`：未完成且关「手动轮询请求上游」时，logs 体无任务状态则降级上游，保证轮询响应带实际 status
- 级联组合缓存仍走组装逻辑；已完成任务路径不变
- 精简：`format_poll_from_log` 内联进 `try_client_poll_from_logs`（单调用点）；status 校验与组装同层；不新增单测

## 2026-08-11 — 官方路由厂商 callback_url 代理
- 上游请求体根级含 `callback_url` 时写入 `plugin_tag.cb`，并改写为系统 `/api/v1/relay/vendor-callback/{id}`（`logs` 主键快查）
- 与入口是否 OpenAI 无关：火山等官方参数可经 OpenAI 路由透传后再识别改写
- `vendor_callback` **只读** logs、不写库；**级联不转发**用户（由后台轮询 S2 结案后通知）；非级联原文转发
- 级联 S2 提交不挂 callback，轮询结案后复用 API 同款处理再转发
- 无回调字段零行为差

## 2026-08-11 — 低余额限制未完成视频：默认关闭 + 档位按单路费用校准
- 默认改为关闭（运营按需开启），避免新建/缺字段时误开拦截
- 默认档按单路约 5～10 元：可用额 &lt;20→1 路，&lt;50→3 路；未命中档位即不限制（不再保留其余档占位）
- 默认关闭时 `max_video_inflight` 直接返回 None，门禁不查在途条数；已保存配置不受影响

## 2026-08-11 — OpenAI 图片响应：Gemini usageMetadata → usage
- `/v1/images/generations` 同步格式化（`build_openai_sync`）补挂 `usage`；轮询路径原已支持
- `usageMetadata`（promptTokenCount/candidatesTokenCount/totalTokenCount）转为 OpenAI `prompt_tokens`/`completion_tokens`/`total_tokens`
- 上游已有 OpenAI `usage` 仍原样透传；仅缺 usage 且有 usageMetadata 时注入，不改其它厂商行为
- 精简：统一 `resolve_client_usage`；Gemini 只认根级 `usageMetadata`；去掉零值包装 `extract_usage`；同步有用量才挂字段；全库无单测残留

## 2026-08-10 — Relay：时区与 HA 分槽缓存（解耦 get_cached_config）
- `get_cached_config` 从 `relay/mod.rs` 迁入 `relay_settings.rs`，拆为独立槽：`get_cached_site_timezone` / `get_cached_ha_enabled`（各 60s TTL，按需查库）
- 热路径只读所需项：计费/额度/任务等只取时区；`calculate_relay_cost` / HA 只取插件开关，避免一次 miss 双查
- 写穿一致性：保存 `site_settings` → `put_cached_site_timezone`；启停 `high_availability_channel` → `put_cached_ha_enabled`；`relay_settings` 仍写穿
- 复查：`put`/`fill_miss` 分离，防并发查库盖掉写穿；API 只依赖 `Database`；时区默认复用 `DEFAULT_TIMEDISPLAY`；`resolve_user_timezone` 改走缓存；HA 插件名常量复用
- 去冗余兜底：缓存层不再二次规范化空时区/空档；空视频档仅在门禁处回落 `RelaySettings::default`；时区空串交给 `parse_timedisplay`
- 再提炼：`CacheSlot` 复用三槽；`setting_value` 复用 settings 读库；门禁非空档零 Default 分配；不留单测
- 清理：去掉重复 `default_video_inflight_enabled`（复用 `default_true`）；前端默认档提为常量；全库无 `#[test]`/`*.test.*` 残留（`ChannelTest` 为产品功能保留）
- 解耦：档位排序/`max_video_inflight` 下沉 `RelaySettings`；缓存层 `get_or_load` 只负责分槽；门禁三行判定
- 复查：门禁改为「入口类别或模型类型含视频」才生效，去掉多余 category 参数，避免仅靠 type_name 漏拦
- 行为与默认值（`Asia/Shanghai`、HA 未装视为关）不变；无单测残留

## 2026-08-10 — 模型调用：低余额限制未完成视频路数 + relay 配置缓存
- 基础设置 → 模型调用设置：可配「可用额低于」金额档与最大未完成视频路数（**0=不限制**），档位可新增/调整
- 超限文案：`当前余额较低，未完成视频任务过多，请等待完成后再试`（与「余额不足」区分）；金额门禁仍只比钱包可用额 vs 本单预扣
- `relay_settings` 整份内存缓存（Arc），保存写穿即时生效；手动轮询开关改读缓存，避免热路径反复查库
- 预扣时机不变（上游成功后再扣），不影响 HA 多子渠失败退款模型
- 复查：COUNT 走 `sqlx::Error`/`?`；档位加载时规范化排序；去掉热路径档位 clone/重复排序

## 2026-08-10 — 创作中心：方案时长可设区间；属性面板隐藏具体计费名
- 模型「参数调整」中 slider（如视频时长）可覆写最小/最大/步长，与方案编辑一致
- 属性配置费率展示隐藏具体规则名，改用通用「计费」标题锚定折扣/峰谷标签

## 2026-08-10 — 清理残留单元测试（keep-system-lean）
- 删除 `settings.rs` 支付渠道 UI `#[cfg(test)]` 模块与 `user_kyc` 归一化单测；业务函数与支付合并逻辑保留
- 自检：仓库无 `#[test]` / `*.test.*` / `cfg(test)` 残留；渠道「测试」页 `ChannelTest` 为产品功能保留

## 2026-08-10 — Relay：手动轮询可优先走 logs，降低上游限流
- 基础设置新增「模型调用设置」Tab（存 `relay_settings`）；开关「手动轮询请求上游」（默认开，兼容现状）
- 开启：未完成任务手动 GET 仍打上游；关闭：优先返回 logs.response_content（复用 `apply_format` 按路由区分 OpenAI/官方），无有效缓存再兜底上游
- 已完成任务缓存短路与后台自动轮询/计费结算不变；抽 `build_client_poll_body_from_log` 复用组装逻辑
- 设置存 `relay_settings`，仅管理端 `/settings/full`，不进入公开接口
- 精简：完成态与「关上游」合并为一条 logs 优先路径（`||` 短路免多余查库）；`json_poll_response` 去重；无效 JSON 用 `?` 早退；不新增单测
- 再解耦：`PollLogView` + `try_client_poll_from_logs` / `format_poll_from_log`，去掉 10 参长列表；级联不再 clone stage；调用点一行短路返回；行为不变
- 再精简：serde 默认复用 `default_true`；失败文案抽取收紧；仓库无单测残留

## 2026-08-10 — 高可用：模型别名分辨率映射 + 映射按钮主题适配
- 子渠道/默认别名支持「高级」按分辨率映射上游模型名（视频 480p…、图片 1k…），存 `config.res_model_mapping`
- Relay `resolve_model` 统一接入：分辨率映射 > 渠道/HA 明文别名 > 模型表别名；计费明细与日志形如 `分辨率映射@480p: a ➞ b`
- 分辨率回退：`子渠该档 → 默认别名该档 → 明文别名`（未填档位不占坑，720p 空则用 `doubao-seedance-1-0-pro` 这类明文）
- 分辨率抽取收敛为 `extract_resolution`：计费特征与模型别名映射共用，去掉重复的 `peek_resolution`
- 复查加固：HA 子配 id 解析复用 `ha_config_id_from_aid`；存库/读入分辨率 key 与后端规范化对齐；兼容历史未规范 key；映射命中日志降为 debug（计费明细仍保留）
- 精简：前端读/存共用 `cleanResModelMapping`；`extract_resolution`/`duration` 按字段按需读取，去掉无节点时的重复扫描
- 精简复用：`isFilled`/`countFilled`/`pruneRecordKeys`/`patchResScope`；结算侧 `mapping_resolution` 统一图片/视频才带分辨率
- 去掉过时 `final_result` 分支（现行方舟为根 `usage` / `content.video_url`；仓库无样例、请求体亦不存在该字段）
- 自检：仓库无 `#[test]`/`*.test.*` 残留；渠道「测试」页 `ChannelTest` 为产品功能保留（按 keep-system-lean 不新增单测）
- 解耦：请求路径统一 `resolve_model_body`（内聚抽分辨率）；结算/测试仍用 `resolve_model(resolution)`；查找档位抽 `res_alias_in_map`
- 性能：仅图片/视频（及 native 图片类）走 `resolve_model_body`；聊天/语音/向量等跳过分辨率解析
- 「收起/高级」按钮去掉 ghost 白边，按明暗主题着色，不影响原有明文别名与 HA 选渠逻辑
- 修复：`native` Gemini 路径 `body` 为 `Bytes`，改用已解析的 `body_json` 做分辨率窥探（消除 E0308）
- 修复：分辨率高级面板按**模型类型**选档（图片仅 1k/2k/4k，视频 480p…），不再误用渠道分类 fallback 混出视频档；已配置历史档位仍可展示清理

## 2026-08-10 — 模型广场：免费张数行价格不再显示 [object Object]
- 根因：有全站折扣时 `formatPrice` 返回 React 节点，却被塞进 i18n `{{price}}` 字符串插值
- 修复：免费行改为「文案前缀 + formatPrice 节点 + unit_per_image」拼接；不抽与 RateDisplay 的大共用层（两侧格式器不同，易引入回归）
- 不新增单测（仓库保持精简，无业务测试残留）
## 2026-08-11 — 门户 cyber_hacker 英雄区光标打字轮播模型名
- 绿色方块光标改为打字机效果：逐字展示下方支持模型名，停顿后删除再切换下一个
- 模型名优先读取页面 `.cyber-model-title`，节奏带随机抖动更自然

## 2026-08-11 — 管理端日志列表用户列展示用户备注
- 使用/任务日志列表：管理员端用户名后直接显示 `admin_remark`（有备注才显示）
- 接口返回 `user_admin_remark`；普通用户响应脱敏不返回该字段

## 2026-08-11 — 日志记录渠道列改为「渠道信息」
- 取消「渠道AID」表头筛选漏斗
- 表头改名为「渠道信息」，单元格展示为 `AID: XXXX`（保留上游标签）

## 2026-08-11 — 用户等级列表行高更紧凑
- 表格改为 `size="small"`，名称列与操作按钮上下间距收紧

## 2026-08-11 — 普通用户列表增加赠送钱包/信控额度排序
- 管理后台普通用户列表排序筛选新增：赠送钱包余额 ↑/↓、信控额度 ↑/↓
- 信控额度排序：仅对额度 > 0 的用户排序（少的在前为 ↑）；额度为 0 的不参与，排在后面
- 筛选栏 Select/Input/按钮字号与高度对齐日志记录（12px / 32px）

## 2026-08-11 — 站点设置增加控制台 Logo 标题链接
- 基础设置 → 站点信息：站点 Logo 下方新增「控制台 Logo 标题链接」
- 控制台侧栏/顶栏 Logo 与站点名可按该链接跳转（支持外链或站内路径）
- 登录页标题链接留空时回退使用控制台 Logo 标题链接

## 2026-08-11 — 高级营销推荐用户列表对齐消费合计筛选
- 推荐普通用户列表：钱包改为消费合计；筛选放搜索框后（排序 + 当月/全部）
- `my-referrals` 返回当月 `current_month_system_cost` / `current_month_gift_cost`

## 2026-08-11 — 普通用户列表消费合计支持全部/当月与排序
- 钱包列恢复「全部数据 / 当月数据」切换：全部用 used_quota，当月走批量消费统计
- 新增 `POST /users/consumption/stats_batch`（usage_daily_stats + 当日 logs）
- 排序筛选增加「消费合计 ↑/↓」（随全部/当月口径变化）

## 2026-08-11 — 普通用户列表钱包展示消费合计
- 列表页系统/赠送钱包不再展示「总充值」，改为「消费合计」（used_quota / gift_used_quota）
- 移除列表页钱包「全部/当月」切换及充值统计批量请求

## 2026-08-11 — 普通用户列表支持按系统钱包余额排序
- 管理后台普通用户列表工具栏新增排序筛选：默认 / 系统钱包余额从高到低 / 从低到高

## 2026-08-09 — 转发规则按站点插件编译/启用来隐藏
- `/forward-rules`：依赖插件未编译或未启用时不返回对应系统默认规则
  - `volcengine_enhance` →「火山方舟 级联视频生成」
  - `asset_manager` →「火山方舟 视频素材转换」
  - `asset_manager_intl` →「火山方舟 视频素材转换(国际版)」「火山方舟 视频素材免审核转换(国际版)」及同 ns 规则
- 与插件中心 `is_plugin_compiled` + `is_plugin_enabled` 一致

## 2026-08-09 — 模型广场首屏只保留一次加载菊花
- 进入 `/home/models`：数据未就绪前不渲染侧栏壳，与 Suspense 全屏菊花衔接，避免「中间菊花 → 热门右侧再菊花」两次加载感

## 2026-08-09 — 侧栏品牌区收拢过渡去挤压
- 模型广场 / 控制台 / API 教程：侧栏收拢时 Logo+站点名改为固定宽度裁切 + 与图标轨交叉淡入，避免文字先挤压再消失

## 2026-08-09 — API 教程页顶栏/侧栏对齐模型广场
- `/docs`（RelayAPI）外壳改为与模型广场相同的 Ant Design `Layout` / `Sider` / `Header`
- 侧栏折叠完全收起（宽度 0，不保留图标轨）；移动端遮罩 + `xs` 断点；折叠状态写入 localStorage
- 顶栏毛玻璃参数与模型广场一致；侧栏收起或移动端时 Logo 显示在顶栏（淡入衔接，避免硬切闪屏）
- 收拢时侧栏/顶栏品牌区固定内容宽度 + 外层裁切，避免站点名被挤压变形

## 2026-08-09 — 在线支付设置重构：渠道列表 / 排序 / 展示配置
- 管理后台「在线支付设置」改为「货币设置 + 支付渠道」两 Tab；原各网关独立 Tab 改为列表整页配置
- 渠道支持排序权重（数字越大越靠前）、启用开关、用户端显示名称 / 副标题 / Logo URL（留空用默认）
- 通联支付合并为单一渠道：后台配置可分别开微信/支付宝；用户端先选「通联支付」，仅双开时再选子方式
- 新增 settings 键 `payment_channels_ui`；公开接口增加 `payment_channels`（含 `allinpay_methods`）
- 用户充值弹窗按后台排序与展示配置渲染
- 下单校验：通联子渠道在 merge 旧拆分配置后再判断，避免未迁移数据绕过关闭状态

## 2026-08-08 — 充值类型多语言：方舟视频消费/退款
- `rechargeTypeLabel` 固定读默认 `translation`（不吃调用方 ns）；修复管理端钱包明细仍显示 `ark_video_*` 原始 key
- 文案/颜色/筛选收敛到 `utils/rechargeType`；扩展类型只改工具与 locale

## 2026-08-08 — 方舟对账：token + 时间双条件精准匹配（统一规则）
- 命中须同时满足：`Count`→整数 token（`47.311`↔`47311` 或裸 token）且创建墙钟（UTC+8）5 分钟槽对齐 ExpenseTime
- 对账入口收敛为 `bill_matches_video(bill, video)`；墙钟/`ceil_5min`/`expense_bounds` 三件套，去掉多层薄包装
- 冲突更新同步刷新 `created_time`，便于延迟对账重匹配

## 2026-08-08 — 方舟视频监控：预扣入账精简与复查加固（与网关相互独立）
- 本插件只按任务合计做 `used−max(charged,流水)` 增量扣/退，不对接 TaskPoller/`logs`
- 估算：tokens 未变则冻结；完整名 `…1-0-pro-fast…`→4.2、`…1-0-pro…`→15；非成功状态金额清零
- 换绑用户/接入点：只写 `wallet_ledger_after` 隔离旧流水，**不改** `used`/`charged`（后台曾清零会导致同 ep 换用户全量重扣；换 ep 由同步重算 + 虚退跳过消化）
- 对账：已确认任务不进匹配池（金额以账单确认为准）；去掉无效的「误标 false 拉回」循环（池内已无 confirmed）
- 查询减压：对账只加载待估算任务的 `raw_response`；已确认仅取 task_id；迁移 `ark_monitor_ledger_after_and_indexes_v1` 合并流水起点列与热点索引
- 入账解耦：`plan_ark_wallet` 纯决策与 IO 分离；预扣/退款/虚退共用同一执行路径，行为不变
- 运维注意：只删 `ark_video_tasks` 而保留流水/锚点时，同步可能不再扣（视为已扣过）；需重扣须同步清锚点与相关流水

## 2026-08-08 — 方舟视频监控：堵住虚退款导致钱包暴增
- 根因：`wallet_charged_quota` 锚点远大于真实已扣流水时，`apply_binding_wallet_delta` 无上限退款（`ark_video_refund`）把差额加回 `balance`（例：1000+600000−1211≈599789）
- 修复：退款封顶为 `max(0, 本绑定流水净扣 − used_quota)`；封顶后仍校正锚点；换绑切断旧流水且不改锚点（见上条复查）
- 展示：钱包「充值」排除 `ark_video_*`；「消费合计」叠加方舟净消费；finance 批量充值统计同步排除
- 已误入账余额需管理员按流水核对后手工拨回（代码不自动改历史 balance）

## 2026-08-08 — 方舟监控：估算预扣 + 对账多退少补（纠正总消费变 0）
- 流程：拉视频估算即预扣；新视频只扣 `used−charged` 增量；分账确认（`is_estimated=false`）后按差额退/补
- 修正：恢复汇总含估算（总消费不再因「只计实价」变成 0）；`1-0-pro` 估算单价 4.2；脏锚点不虚退；锚点已=used 但流水仍多扣时补退

## 2026-08-08 — 方舟对账：仅精准匹配才确认，取消错误分摊改 false
- 根因：`updated_at` 漂移导致 Count=47.311(千token)=47311 匹配失败；失败后比例分摊把多条视频都标 `is_estimated=false`，钱包按错误合计扣费
- 修复：按创建时间升序 + ExpenseTime 窗口/5 分钟对齐精准 1:1；只有命中才 `is_estimated=false`；未匹配保持估算、不再分摊写库；DB 重建带 `total_tokens`

## 2026-08-08 — 方舟估算单价：恢复 1.0-pro=15 / pro-fast=4.2，新增 2.5=70
- `doubao-seedance-1-0-pro`→15、`1-0-pro-fast`→4.2（先匹配 fast）；`doubao-seedance-2-5`/`2.5`→70（在线·输入无视频）
- 复查：退款入账抽 `apply_ark_system_refund`；误标 false 仅在有未核销账单时拉回估算态且不改金额；匹配优先待对账再按创建时间

## 2026-08-08 — 新增模型折扣限价默认开启
- 管理后台添加模型时，「折扣限价」默认开启，倍率默认 `1.0`
- 后端创建接口缺省值与库列 DEFAULT 同步为开启 / `1.0`（不影响已有模型）

## 2026-08-08 — 用户实名认证 KYC
- 注册设置新增「开启用户实名」开关（`enable_user_kyc`）
- 管理后台普通用户编辑新增「用户实名」Tab：个人/企业证件、有效期、审核状态
- 个人中心在开关开启后展示账号实名认证；用户可提交，管理员可录入与审核
- 新增表 `user_kyc`；证件上传走系统 TOS（`/user/kyc/upload`）

## 2026-08-08 — 去掉 ve-tos-rust-sdk，TOS 全走 reqwest 0.12 + TOS4
- 删除 `ve-tos-rust-sdk`（及其 reqwest 0.11 / hyper 0.14 双栈）；`services/tos.rs` 共用 `signed_request`
- 公开 API 不变：连通性 / 上传（x-tos-acl + x-tos-tagging）/ 删除 / 标签 / 列目录 / 预签名
- 修复：去掉 SDK 后丢失传递的 `tokio/rt-multi-thread`（tokio 已 `default=[]`）；`Cargo.toml` 仅补该特征（+原有 fs/signal），避免 `#[tokio::main]` 编译失败且不扩大特征面
- 签名对齐 SDK：Authorization 恒 empty payload hash；SignedHeaders 仅 host / content-type / x-tos-*
- 创作中心删项目：先取 `tos_object_key` 再删库；`purge_prefix` / `spawn_purge` 汇总真实成功失败（不再吞错）
- 精简：无 `#[test]`/`*.spec` 残留可删；去 assets 孤儿注释；TOS4 HMAC 复用；收紧 purge 模块可见性
- `signed_request`：网络错/408/429/5xx 最多再试 2 次（100ms 起短退避）；公开 API 与预签名不变
- TOS HTTP Client `OnceLock` 复用；重试循环去不可达尾部分支
- 修复创作中心删项目 TOS 漏删：删库前用项目 uid + object_key/file_url；404 不计 deleted_ok
- 修复 ListObjects `SignatureDoesNotMatch`：query 用 `Url::query_pairs` 编码，签名与发送同一字符串（含 tagging）
- 复查完善：删项目取资源 key 失败不再 `unwrap_or_default` 吞错；`list_folder` 续页；purge 去重后并行删；标签 PUT 补 `Content-Type: application/json`
- 再精简：删项目依赖 assets `ON DELETE CASCADE` 去掉重复 DELETE；TOS `request_host_path` 合并 Path/VHost；HMAC 少分配；list query 零拷贝字母序
- 去掉无用兜底：ListObjects 不再 JSON 回退 / `<Code>` 启发式；ListBuckets JSON 仅 `{` 体且单路径 `Buckets[].Name`

## 2026-08-07 — 级联 720p←480 裁剪开关 crop_480p
- 转发规则新增 `crop_480p`（缺省 true）：仅目标 720p 且底座 480p 时是否 MediaKit 居中裁剪；false 跳过
- 其它分辨率不受影响；现网未配置规则保持裁剪行为不变

## 2026-08-07 — POLL_FAIL_LIMIT=15（防漏终态成功）
- TaskPoller / `poll_task_result` 连续失败上限改为 15，上游短暂抖动时不轻易退款或放弃
- 倒序休眠 5→1s、级联裁剪/抽帧与增强路径不变

## 2026-08-07 — 复查：cascade_resolve_enhance key 只规范化一次
- `cascade_resolve_enhance` 分辨率 key 只规范化一次（语义不变）

## 2026-08-07 — 清理根目录一次性探测/部署脚本（无产品依赖）
- 删除含硬编码 SSH 凭据的孤儿脚本：`check_*.py` / `auto_deploy.py` / `*_deploy*.sh|exp` / `find_project*` / `upload_fileio.py` / `final_install.sh` 等
- 正式路径仍用 `deploy.sh` / `export-images.sh` / `push-images.sh` / `dev.sh`；产品「通道测试」保留；业务单测仍为 0

## 2026-08-07 — 轮询/级联再精简（无行为变更）
- 删单调用薄包装：`cascade_ai_enhance_allowed` / `cascade_s1_480p_crop_rect` / `cascade_root_str` / `cascade_json_raw_str`；延迟公式并入 `poll_wait_before_query`
- 裁剪判定并入 `cascade_ensure_standard_480p_video`；文档旧名 `cascade_mediakit_tool` 更正为 `cascade_mk_url`

## 2026-08-07 — 轮询/级联 MediaKit 参数收口（无行为变更）
- `PollTaskOpts` 收口 `poll_task_result` 可选参（缺省 300s，MediaKit 用 Default）
- `CascadeMk` 捆 `(http, ch, auth_type)`；裁剪/抽帧共用；删薄包装 `cascade_target_resolution` / 单调用 `processing_json_from_submit`

## 2026-08-07 — 任务轮询倒序休眠 5→1s（POLL_FAIL_LIMIT 共用）
- `poll_task_result`：每次查询前休眠 5→4→3→2→1s；连续失败改用共用 `POLL_FAIL_LIMIT`（现为 15）
- 级联裁剪/抽帧（`cascade_mk_url`）状态轮询共用；阶段二增强仍由 GET/TaskPoller；素材转换 / POST 提交重试不套用

## 2026-08-07 — 级联 MediaKit 收口为 cascade_mk_url（无行为变更）
- 裁剪/抽帧共用 `cascade_mk_url(..., out_ptr)` 直接取产物 URL；抽帧逻辑并入 `cascade_on_s2_succeeded`
- 去掉中间 JSON 返回层与单用 attach helper

## 2026-08-07 — 级联/响应路径可见性收口（无行为变更）
- `cascade.rs` 仅模块内使用的 helper 降为私有；`push_unique` 改为文件内私有
- 全库无 `#[cfg(test)]` / 业务单测残留；产品「通道测试」页面保留

## 2026-08-07 — 级联增强 S2 成功后按需刷新尾帧图
- S1 有尾帧时对 S2 `/result/video_url` 抽尾帧，仅写入 stage2 `last_frame_url`（落库 stage1 保持原图）
- 客户端 / 用户端经 `cascade_s1_with_s2_url` 叠尾帧；GET/后台共用 `cascade_on_s2_succeeded`
- MediaKit 工具收口 `cascade_mk_url`（曾用名 cascade_mediakit_tool）；抽帧失败软降级

## 2026-08-07 — FailBill 工厂 + spawn_protected（含 native，保行为）
- `FailBill::transport` / `http` / `biz` + 短链，六模态（含 native）park 字面量收口
- `spawn_protected` 收 oneshot 连接保护样板；native/chat 计费短 spawn 仅清晰命名，不强套 oneshot

## 2026-08-07 — TaskPoller 周期改为 30 秒
- `POLL_TICK_INTERVAL_SECS`：15 → 30；活跃窗口 / 分批 / 失败退款逻辑不变，客户端主动 GET 仍即时

## 2026-08-07 — relay spawn 捕获写法收口（保连接保护）
- 保留 `tokio::spawn`（客户端断开仍完成上游/预扣/落库）
- 去掉 `_c` 再赋值；改为 spawn 块内一次 `clone` 成最终变量名（video/audio/generic/image/chat）

## 2026-08-07 — 精简：去掉 playground_2026 残留单测
- 删除 `standalone_path_tests`（`#[cfg(test)]`）；保留产品功能「通道测试」页面；业务路径不变

## 2026-08-07 — HA 实现再收口（保行为）
- 去掉首败上多余的 category/billing 快照；续试退预扣收成 `refund_continue`
- 保留：末次强制停切、预扣清零防双退、首败 `endpoint`、chat 预扣走 `on_access_err`

## 2026-08-07 — HA 终态正确性修补（行为对齐首败）
- 末次尝试强制停切并在 `fail` 内 settle；续试退预扣后清零首败预扣，杜绝 finish 双退
- 首败 `endpoint` + `FailBill.prefer_status`/响应贯通 settle；chat 预扣走 `on_access_err`+`break`
- 不另造 audio 透传专用 HA API（原生 TTS 仍 Ok→`ha.ok`）

## 2026-08-07 — HA API 解耦精简（行为不变）
- 首败合一 `FirstFail`；`settle_first` 取代 `bill_first`；`HaAttempt::park` 合并 stash+err_of
- `HaBillCtx::new` + `category`/`billing_model`/`db` 链式补参，端点不再堆满可选字段字面量

## 2026-08-07 — 修复 HA 终态落库丢入参/上游体/响应
- 保留首败 `FailBill` 至环结束写入；禁止把 `upstream_url` 误写入 `upstream_req_content`；无账单时响应用首败错误体

## 2026-08-07 — 去掉未使用的 record_zero_cost_upstream_fail
- HA 已统一 `FailBill`/`err_of`/`ha.fail`，该薄封装无调用方；保留 `record_zero_cost_fail` 终态/业务侧记账

## 2026-08-07 — HA 记账参数收口为 HaBillCtx
- `fail` / `finish` / `bill_first` 共用 `HaBillCtx` 命名字段；去掉 `FirstUpstreamFail` 无用 channel 字段；`ok` 的 url 改为 `&str`

## 2026-08-07 — HA 插件日志路径精简
- 选渠注入子配 `name`，snap 不再逐次查 `channel_configs`；`ok` 内含 `save`；去掉 FailBill 无意义 clone
- `get_ha_logs` 合并重复 SQL；语义不变：中间失败不写 logs，环结束一条 `ha_usage_logs`

## 2026-08-07 — HA 一条日志 + 插件收口
- `set_pending` 禁止 `None` 覆盖；六模态统一复用同一 pending，杜绝重试再 INSERT
- `finish` 异步收口：pending 仍为处理中时补记首败并写 `ha_usage_logs`（修复选渠耗尽/异常退出丢插件日志、主站卡「处理中」）

## 2026-08-07 — HA 插件日志仅记真实 HA 渠
- `ha_usage_logs` 写入条件改为实际命中 HA 组/子渠；令牌开 HA 但走物理渠不再入库

## 2026-08-07 — HA 终态落库 + 插件使用日志
- HA 中间失败不再反复 UPDATE `logs`，环结束一次记成功/首败；非 HA 行为不变
- 新增 `ha_usage_logs`（关联 `logs.id`，`attempts` JSON 含子渠错误/URL/YID 等）及插件「使用日志记录」Tab
- 去掉 `reinstate_first_log`；上游失败收口为 `HaAttempt::fail` / `save`

## 2026-08-07 — 根目录精简：去掉冲突修补脚本与本地杂物

- 删除 `fix_conflicts.sh`（一次性冲突标记剥离，非产品路径；开源打包排除项同步去掉）
- 清理本地已忽略的 `*.log` / `.DS_Store` / 空 `opensource/`；不删 `backup/` 大备份与运维部署脚本

## 2026-08-07 — 合并 origin/cgdev0726 到 chenzs

- 保留本分支 MediaKit `volcengine_enhance_logs` 关联表与日志恢复
- 合入对方创作中心 2026 工作流/作品/生成图视频、导出脚本与导航等改动；迁移块并存互不覆盖

## 2026-08-07 — 精简复查：无单测残留；MediaKit 日志 UI 轮询复用

- 全仓确认无 `#[cfg(test)]` / `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）
- `VolcengineLogs`：数据恢复状态轮询合并为 `beginRecoverPoll`，去掉重复 interval 逻辑

## 2026-08-07 — MediaKit 关联日志小收口（行为不变）

- 列表 FromRow 去掉未查询的大字段占位；回填 keys 只补 `model_id`；空结果免二次 COUNT；回填 INSERT 去掉与 NOT EXISTS 重复的 ON CONFLICT

## 2026-08-07 — 去掉 MediaKit 无用的 plugin_tag 根 mid

- `video.rs`：不再写入 `plugin_tag={"mid":…}`（列表已改关联表，业务不读根 mid）；保留快乐小马 / 级联 `cascade` 标记
- 关联写入、回填、计费与展示路径不变

## 2026-08-07 — 火山 MediaKit 使用日志改为 logs.id 关联表

- 新建 `volcengine_enhance_logs`（主键 `log_id` = `logs.id`）；列表 COUNT/分页走窄关联表再 JOIN，避免大表 `model=ANY` 扫全库
- 写入：`video` pending 落库后对 `volcengine_media_enhance` 幂等关联
- 插件「使用日志记录」Tab 增加「数据恢复」：后台分批回填历史 logs（按预置 mid/model_id），不阻塞页面

## 2026-08-06 — 级联 cgt 复查修复（行为对齐、无多余超时）

- `force_json_task_id`：根已有 `task_id` 时与 `id` 同步为对外号（对齐 `find_id` 优先级，避免 bill/OpenAI 提交态仍落到上游号）
- 级联 S1 中间态/失败落库同步 cgt；用户日志脱敏仅在真实级联时强制改写 id
- `cascade_apply_processing_status` 复用 `force_json_task_id`；裁剪轮询超时保持 **300s**（撤回误扩到 600）

## 2026-08-06 — 精简复查：无单测残留；级联 helper 可见性收口

- 全仓确认无 `#[cfg(test)]` / `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）
- 级联：仅模块内使用的 helper 曾改为私有；其后 `cascade_json_raw_str` / `cascade_processing_json_from_submit` 已删除（逻辑并入调用方）

## 2026-08-06 — 级联对外任务号 cgt-xx-xx（隐藏上游 S1 id）

- 对外 id：`cgt-{YYYYMMDDHHmmss}-{5位随机}`；S1 真 id 仅 `plugin_tag.cascade.s1_task_id`（scrub 保留；用户 plugin_tag 白名单不含 cascade）
- POST 改写响应体 `id` 后按原规则落库；S1 轮询成功写入 `response_content` 时再把 `id` 换成同一 cgt
- 取消接口不做级联特殊处理

## 2026-08-06 — 异步轮询响应加速（功能不变）

- TaskPoller 周期 30s → **15s**，无人主动 GET 时更快结案 / 触发级联 S2
- `poll_task_result`：**立即首查**再 2s→5s 递增间隔（原先睡再查，同步图/裁剪/通道测试更早看到终态）
- 级联裁剪 POST 临时错：固定 10s → **2→4→8→10s**
- Playground 轮询 5s → **3s**；次数上限按比例提高，软超时墙钟仍约视频 60min / 图片 15min

## 2026-08-06 — 级联 S2 临时错重试改为递增短退避

- `cascade_stage2_submit`：可重试错误仍最多 5 次、可恢复码不变；等待由固定 120s 改为 **10→20→40→60s**（总睡眠约 130s，原约 480s）
- 临时限流/上游抖动仍可在重试窗口内成功提交；单次等待更短，缩短级联墙钟

## 2026-08-06 — 创作方案：补齐 OpenAI 图片/视频系统预设

- 图片：强化系统方案 `gpt_image_2` 为「OpenAI 图片生成方案」，覆盖文生/图生常用参数（含 style、response_format、transparent 背景等）
- 视频：新增系统方案 `openai_video`（OpenAI 兼容/Sora），预设 size、duration、resolution、ratio、watermark；图生参考图仍由画布素材写入 `images`
- 仍由 `load_schemes_from_db` 自动合并 `is_system` 种子，无需手工迁移；重启后端后管理端可见

## 2026-08-06 — relay 日志级别与素材转换小收口

- `relay` 下原 `tracing::debug!` 改为 `info`（常规跳过/进度）或 `warn`（选渠预设缺失）
- `asset_convert`：插件不存在/未启用合并为同一跳过分支

## 2026-08-06 — ha.rs 轻量收口（行为不变）

- 常量上移归并；`HaTimeoutCtx::remaining` 复用剩余预算计算；`resolve` 用 filter/map 收紧
- 去掉仅调用一次的 `push_exclude`；`begin` 日志直接用已算好的 `budget_secs`
- 按仓库精简规范：不新增留存单测

## 2026-08-06 — 精简：无单测残留；去掉超时薄封装与单测措辞

- 全仓确认无 `#[cfg(test)]` / `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）
- `http_client`：合并 `upstream_timeout` 薄封装进 `upstream_timeout_duration`；删除已无调用的 `with_upstream_timeout_if`
- `is_asset_api_enabled` 注释去掉「便于单测」措辞（业务函数，非测试入口）

## 2026-08-06 — HA 墙钟预算：嵌套转发避免 504 吞掉上游真实错误

- 根因：深站 HA 在首败（如火山 `InternalServiceError`）后继续切子渠，墙钟被入口 Nginx（常见 600s）切断，中游只收到 504 HTML，无法透传首败 JSON
- `HaAttempt` 增加整次墙钟预算（仅 failover 开启；`ha_total_timeout_secs`，0=自动 `min(540, 上游超时-60)`）：预算耗尽立即 `finish` 首败；首次尝试仍用全局上游超时
- 备渠超时：`HaTimeoutCtx::resolve()` 在真正 send 前按剩余墙钟重算（避免 image transform 等前置耗时把超时算得过宽）
- 非 HA / 未开 failover 不启用预算，行为与原先一致；多厂商 HA 遇短暂 502/504 仍可按预算切换
- 去掉已无调用方的 `with_upstream_timeout_if`；下载超时复用 `with_timeout`
- 插件配置/后台 UI 增加「HA 整次墙钟预算」；黑名单提示补充 `InternalServiceError` 等平台级错误码

## 2026-08-06 — 清理测试残留与仅测用可见性

- 确认全仓无 `#[cfg(test)]` / `#[test]` / `*.test.*`（保留产品页 ChannelTest）
- `period::quota_day_key_with_cutover_at`、`BillingRule::multiplier_at_local_time` / `time_multipliers_enabled` 改为模块内私有（原为单测入口）

## 2026-08-06 — OpenAI 响应 status 约定收口

- 异步轮询/结案（`is_poll=true`）：一律 poll 骨架，带正确终态 `status`；`error`+成功类 → `failed`
- 异步 POST 提交 ack：仅当上游响应自身有 task_id 时输出 `status:pending`（勿用 log_id 冒充）
- 同步成功：无 `status`；同步/POST 业务错误为纯 `error` 体
- 级联处理中响应额外去掉 `data`，避免误带出产物 URL
- 参数由易混的 `is_async_submit` 更名为 `is_poll`；`resolve_created` / poll 内一次 `find_urls` 复用

## 2026-08-05 — HA finish 保留首败头；合并 upstream_fail

- HA `finish()`：业务侧原样；全失败优先返回首次 `UpstreamHttpError`（保留响应头），兜底按 `first_fail` 重建
- `upstream_fail` / `upstream_fail_with_headers` 合并为三参 `upstream_fail(status, msg, headers)`

## 2026-08-05 — 合并 record_zero_cost_upstream_fail 双入口

- 去掉 `record_zero_cost_upstream_fail_hdr`，统一为 `record_zero_cost_upstream_fail(p, headers)`；无头传 `None`

## 2026-08-05 — 清理无用单元测试模块

- 删除 `time_system/period`、`models/settings`、`models/model` 内 `#[cfg(test)]` 模块；业务逻辑不变

## 2026-08-05 — Relay 透传上游响应 Header

- 新增 `relay/upstream_headers`：过滤 hop-by-hop / body 绑定头后，将上游诊断头（如 `x-request-id`、`x-client-request-id`）挂回客户端
- 覆盖 chat / responses / image / video / audio / generic / native / SSE 成功路径；上游 HTTP 失败经 `UpstreamHttpError` 可选携带同源头
- 不转发 `content-length` / `content-encoding` / `content-type`（body 由网关重建），避免与计费改写、解压冲突
- 提炼 `is_sse` / `is_stream_content_type` / `header_str` / `with_content_type`；SSE 用 insert 覆盖 Cache-Control，避免重复头
- 修复 HA `finish()` 经 `upstream_fail` 重建导致丢失上游响应头；`AppError::IntoResponse` 一次匹配拿走 HeaderMap 避免 clone

## 2026-08-05 — 后台 TaskPoller 周期 120s → 30s

- 未结算异步任务自动检查间隔由 2 分钟改为 30 秒（`POLL_TICK_INTERVAL_SECS`），加快结案/退费；分批上限与失败计数逻辑不变

## 2026-08-05 — apply_format 去掉未用 pool 并改为同步

- 确认 `apply_format` 的 `_pool` 从未使用且体内无 await：去掉 pool，改为同步 `fn`
- 连带 `format_async_task_failed` / `ensure_client_async_failed` / `cascade_s2_client_processing` / `cascade_format_s2_succeeded` 同步化并去掉仅透传的 pool；调用点去掉无意义 `.await`

## 2026-08-05 — task 日志查询内联单次 helper

- `load_task_relay_log_by_task_id` / `load_task_relay_log_by_id` 仅各一处调用，内联到 `task_status` / `sync_single_task`；共用 `TASK_RELAY_LOG_COLS` / `format_task_relay_sql` 保留

## 2026-08-05 — 级联 720p/1080p 底座可同档

- 目标 720p：底座可选 `480p`/`720p`（默认 480p）；1080p：可选 `720p`/`480p`/`1080p`（默认 720p）；2k/4k 不变
- 前后端 `BASE_OPTIONS` / `cascade_allowed_bases` 对齐；480→720 裁剪仅在底座实为 480p 时触发

## 2026-08-05 — 级联 480p 非大模型超分改用 resolution_limit

- 级联阶段二：目标 480p 且非大模型（非 vve-gt）时，MediaKit 入参用整型 `resolution_limit=480`、不传 `resolution`，锁定标准 480p 增强
- 其它目标分辨率仍传字符串 `resolution`；`build_volcengine_media_enhance_body` 透传 `resolution_limit`

## 2026-08-05 — 异步失败约定：200 + status:failed；轮询 500 可重试

- 约定：异步业务结案失败对外 HTTP **200** + body **`status:failed`**（含 id/error）；传输层临时错仍非 2xx
- `format_openai` / `is_failed_task_status` / `format_async_task_failed` / `force_json_task_id` 收口 `response_formatter`
- 级联：`cascade_stage2_submit`、`cascade_resolve_s2_poll`、`cascade_format_s2_succeeded` 迁入 `cascade.rs`；InflightGuard 私有化；GET/后台 `pick_poll_target` 共用
- live/缓存失败统一 `ensure_client_async_failed`：上游无 status 时补齐，已有 `status:failed` 则保留
- `tencent_aigc_task` 收口 ErrCode/Message/FileInfos/TaskId；`is_poll_transport_retryable`（含 500/502）供轮询 / S2 POST / 裁剪复用

## 2026-08-05 — 高可用：参数上限放开 + 报错停止切换黑名单

- Failover 参数：去掉最大切换次数上限（原 1～100）与冷却秒数前端下限（原部分 min=5）；仅保留次数 ≥1、秒数 ≥0
- 新增「报错信息停止切换黑名单」：错误信息命中关键词后立即停止备渠 Failover（不熔断），避免同源业务错误空耗；白名单仍为「跳过熔断、可继续切换」；缺省空列表，首次保存时写入 `plugin_configs`
- 名单关键词：保存/加载时 trim、去空、按小写去重，避免空白词误命中；内存侧去掉多余 `Arc`（`AppState` 已包一层）

## 2026-08-05 — 级联超分：大模型增强仅限 720p/1080p/2k

- 转发规则管理页：增强选项中「大模型」仅在 720p、1080p、2k 可选；480p/4k 不展示且解析时回退标准
- 后端 `cascade_resolve_enhance`：对不允许分辨率上的 `ai` 配置兜底为标准版，避免 JSON 直配绕过

## 2026-08-05 — 级联 S2 解析/出参拼装收口（无行为变更）

- `cascade_s2_parse_post_200` / `CascadeS2Post200Fail`：HTTP200 有无 task_id、业务错分类收口；文案与状态码仍在 `task.rs`（不引入 cascade↔proxy 耦合）
- `cascade_upstream_req_combined`：成功/失败共用 stage1+stage2 出参拼装；裁剪提交分支略收紧，语义不变
- S2 终态退款前统一 `normalize_error_http_status`：HTTP 4xx/5xx 原样落库；非 HTTP 业务码经 `infer_error_status_code` 转换后再规范化，与日志 `status_code` / 轮询失败路径一致
- 裁剪判定：目标仍仅 720p；ratio 允许 S1 缺省时从上游出参/用户入参补齐；不误用入参 `resolution=720p` 挡裁；目标分辨率解析收口 `cascade_resolve_target_resolution`；非 200 错误文案统一 sanitize

## 2026-08-05 — 峰谷时段倍率：开关生效、请求开始锁定

- 后端仅在 `enable_time_multipliers=true` 时应用 `time_multipliers`；关闭开关后脏数据不再计费
- 倍率在请求开始（取规则时）锁定；写入 `billing_features.time_multiplier`，异步结算优先用快照，避免长任务跨峰谷变价
- 计费明细与金额一致：已乘过价不再重复追加「时段倍率」文案；渠道测试预览同步展示峰谷倍率
- 管理端：开启峰谷时须至少配置一条有效时段；说明文案同步为「请求开始锁定」

## 2026-08-04 — 级联 S2 失败透出真实错误并落库 stage1/stage2 出参

- 阶段二 HTTP200 无 task_id：优先识别上游业务错误体并透出真实 message（不再笼统「提交成功但未能解析」）
- 阶段二失败时同样写入 `upstream_req_content={stage1,stage2}`（尊重 enable_log），避免日志出参只剩阶段一请求体被误判为「丢了 stage」
- 无结构化错误时 warn 截断响应体便于排查；裁剪策略不变（仅 480→720）

## 2026-08-04 — 上游日额度自定义刷新时刻与冷却

- 上游渠道配置支持配置每日刷新时间点与冷却分钟（有效刷新 = 时间点 + 冷却）
- 日额度旁齿轮弹层编辑；选渠/扣费/退款按站点时区切点懒重置；默认 00:00 / 冷却 0 行为不变
- 日键算法回退多天，正确处理冷却跨午夜；迁移：`channel_configs_daily_reset_cutover_v1`
- 刷新时间点按站点默认时区（非系统运行时区）；开启「显示时区后缀」时标签追加 `(UTC+8)` 一类标记
- 刷新设置改为独立弹窗（保存/取消）；日额度旁直接展示刷新时刻与冷却摘要

## 2026-08-04 — 上游渠道配置弹窗布局紧凑化

- 添加/编辑上游渠道配置弹窗改为多列紧凑布局：名称与服务商、分类与排序/状态、倍率/优先级/权重与额度开关同行
- 冗长 `extra` 说明改为标签 Tooltip；表单 `size="small"`、收窄宽度与间距

## 2026-08-04 — 异步轮询临时错可重试 & 级联 480→720 裁剪收口

- GET 与后台轮询对临时 HTTP（429/502/503/504）及连接失败统一可重试，避免一次轮询失败立刻退费导致上下游账不一致
- 业务终态仍立刻退费；成功轮询清零 `POLL_FAIL`；满限后若仍冻结则幂等补退
- 错误 HTTP 状态统一 `normalize_error_http_status`（仅 4xx/5xx，其余 502）；文案兜底识别 `too many requests`
- 级联：仅 480→720 且 S1 为 480p+16:9/9:16 时 MediaKit 居中裁标准 480p；480→480 等跳过

## 2026-08-03 — 腾讯云视频转发兼容可灵官方 contents/settings

- 仅 `tencent_vod_video`：可灵新协议经腾讯云转发时，从 `contents[{type:prompt}].text` 提取 `Prompt`，避免上游 `Prompt cannot be empty while FileInfos is empty`
- `settings.resolution` / `duration` / `aspect_ratio` / `audio` 映射到 `OutputConfig`；`contents` 中首尾帧/参考图/参考视频映射到 `FileInfos`（及 `LastFrameUrl`）
- 用户已传腾讯云原生 `FileInfos` / `OutputConfig` / 顶层 `prompt` 时仍原样优先；`tencent_vod_image` 与旧扁平字段路径不变
- 结构：`tc_collect_video_src`（单次扫 contents）→ `tc_build_video_file_infos` / `tc_build_video_output_config`；主函数只做装配，便于后续扩展 contents 类型或 settings 字段

## 2026-08-03 — API 教程隐藏上游实现细节

- 常用调用示例与协议文档（DocsApi + SitePortalPro）去掉转发路径、协议转译、上游字段映射、渠道切换等内部说明
- 保留用户侧端点、参数与示例；管理端渠道/模型配置指南未改；「重置初始化」后生效

## 2026-08-03 — 模型广场输入图免费文案

- `minimax_h3` / Seedream 明细输入图改为「N 张以内免费，超出部分 单价/张」；分辨率阶梯等其余展示不变

- `contents` / FileInfos 单次扫描；SSE 行解析、媒体字段判定、Option 合并抽公共辅助
- `ExtractedFeatures` 改 derive Default；`merge` 去多余 clone；计费语义不变

## 2026-08-03 — API 教程去掉内部实现细节（可灵 / MiniMax）

- 可灵视频/生图、MiniMax 图/视频调用文档去掉 `target_type`、上游路径、JWT/密钥形态、字段映射等后台说明
- 保留端点、示例与用户侧参数；管理端「重置初始化」后生效

## 2026-08-03 — 可灵 3.0 / 输入图计费审查修复

- 计费：新可灵特征只认 `settings.resolution`（及 sound/contents 图数）；`kling_video` 结算在无 mode 时用 resolution 兜底映射 std/pro，特征侧不再写 mode
- 请求体：`kling_video` 仅新协议（`contents`/`settings`/`options`，及 `images`/`image_urls`→contents）；不读旧可灵 `image`/`image_tail`；空 `contents` 不误切 image-to-video
- 特征：`image_ref_count` 计入 `contents` 图条目，避免「视频秒价+输入图」漏计

## 2026-08-03 — 清理 date_helper 遗留单测

- 删除 `date_helper.rs` 中 `model_detail_days` 的 `#[cfg(test)]` 模块；业务函数不变
- 全仓复查无 `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）

## 2026-08-03 — 输入图免费张数可配置（Seedream / 视频秒价+输入图）

- `volc_seedream_pro`：新增 `extended_config.free_image_count`（新建表单默认 2）；未配置的旧规则结算仍按首张免费，避免改价
- `minimax_h3`：展示名改为通用「视频秒价+输入图」（`billing_rule` 键不变）
- 模型广场 / RateDisplay：按 `free_image_count` 动态显示「第 N 张起」，与结算一致；免费张数解析抽到 `billingFreeImages` 共用

## 2026-08-03 — 可灵视频 3.0 推荐协议（kling_video）

- 新增 `target_type=kling_video`，与旧 `kling`（`/v1/videos/*`）解耦；旧规则与映射零改动
- 系统规则仅 2 条：文/图 `/text-to-video/${model}`（body 含非空 `contents`→`/image-to-video/${model}`）、Omni `/omni-video/${model}`；`poll_path=/tasks?task_ids=${task_id}`
- Body 组装为官方 `contents`/`settings`/`options`（文生顶层 `prompt`，图生/Omni 才把 prompt 放入 `contents`）；不兼容旧可灵扁平 `image`/`image_tail`/`mode`- 路径占位与现有规则一致用 `${model}`；`resolution`/`audio` 直读新字段
- 轮询结果不改写 body：按「按任务ID查询」解析 `data[0].id` / `status` / `outputs[].url`（旧 `data.task_result` 仍兼容）
- 鉴权：`kling_video` 渠道密钥填官方 API Key，`Authorization: Bearer` 直传（不再签 JWT）；旧 `kling` 仍 `access_key:secret_key` → JWT
- 计费：新协议靠 `settings.resolution` + 结算侧无 mode 时的 resolution→std/pro 兜底；`settings.audio`→sound on/off
- 重启后端执行迁移 `kling_video_v3_forward_rules_v1`；模型绑定推荐规则并将 model_id 设为官方路径段（如 `kling-3.0` / `kling-3.0-omni`）

## 2026-08-03 — 合并 origin/cgdev0802 到 chenzs

- 保留本分支：HA 子渠同档权重随机、MiniMax 图片/视频转发与文档、`rule_type` 统一为 `minimax`
- 并入 cgdev0802：兑换码单用户活动次数限制与防刷、渠道配置分类、顶栏毛玻璃与开源打包/多实例编译隔离等；双方功能并存

## 2026-08-03 — HA 子渠恢复同档权重随机分流

- HA 组内子渠改回：最高 `priority` 档内按 `weight` 比例随机（权重越大命中率越高），与物理渠选路一致
- 抽共用 `pick_weighted_by`（负权重按 0），去掉确定性「weight 降序 + 绑定序」；failover / 熔断 / exclude 语义不变
- 复查：子渠选择去掉多余 clone；补 `minimax_forward_target_type_restore_v1`，防止误把 `target_type` 写成纯 `minimax` 时转发失效；帮助文案区分 `rule_type=minimax` 与 `target_type=minimax_image|video`

## 2026-08-03 — MiniMax 图片/视频调用教程

- DocsApi「2.常用调用示例」新增 `minimax-image` / `minimax-video`（中英文种子文档）
- 覆盖文生图、主体参考/图生图、`image-01-live` 画风；文生视频、首尾帧图生、多图参考、图+视频+音频参考及轮询
- 门户 SitePortalPro 同步同名种子；管理端「重置初始化」后生效

## 2026-08-03 — MiniMax 转发 rule_type 统一为 minimax

- 图片/视频两条规则的 `rule_type` 统一为 `minimax`，筛选标签合并；`target_type` 仍分别为 `minimax_image` / `minimax_video`，转发逻辑不变
- 迁移 `minimax_forward_rule_unify_v1` 回填已有规则；重启后端以执行迁移

## 2026-08-03 — 清理测试残留与无用文件

- 删除 `auth.rs` 中遗留的 `#[cfg(test)]` IP 黑名单单测模块（业务 `check_ip_blacklist` 路径不变）
- 删除根目录无用 `test.txt`；全仓复查无 `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）

## 2026-08-03 — MiniMax 图片生成转发规则

- 新增系统转发规则 `MiniMax 图片生成`（`target_type=minimax_image`）：`/v1/images/generations` → `/v1/image_generation`
- 支持 OpenAI 兼容参数与官方参数混用：`size`/`ratio`→`aspect_ratio`，`watermark`→`aigc_watermark`，`b64_json`→`base64`，`image`/`image_urls`→`subject_reference`
- 官方字段（`aspect_ratio`/`subject_reference`/`prompt_optimizer`/`style`/`seed`/`n`/`width`/`height` 等）原样透传，官方优先
- 响应适配：`data.image_urls` / `data.image_base64` 归一化为 OpenAI `data[]`；识别 `base_resp.status_code` 业务错误
- 修复同步响应带 `id`+媒体时误走轮询的问题（MiniMax 等同步生图）
- 渠道 `minimaxi.com` + 图片类别无规则时自动推断 `minimax_image`
- `size_to_ratio` 统一为 8 档最近邻 + 解析失败回退原串，腾讯云与 MiniMax 共用，去掉重复实现
- 成功张数优先取 `metadata.success_count`（兼容整型/数字字符串），再回退 usage / 数组计数

## 2026-08-03 — 出站 HTTP 超时遗漏修补

- 修复 image 流式误挂 1800s 总超时（与 chat/native 对齐，避免长 SSE 被切断）
- 补齐渠道测试、文档翻译、数据同步、图标同步、Playground 下载、素材上游等 `http_client` 请求超时
- 抽 `with_upstream_timeout_if` / `with_download_timeout` / `download_bytes`，下载超时统一默认 200s

## 2026-08-03 — 兑换码：单用户活动参与次数限制

- 生成兑换码新增独立开关「限制单用户参与本活动次数」（默认不限）；开启后可设 N 次，按活动名跨码累计
- 与「兑换码支持多次兑换」解耦：前者约束活动参与，后者只约束单码可兑总次数
- 兑换接口防刷加固：限流前置、避免全量读配置、无效码事务外快速失败、活动统计索引、顾问锁防并发超限
- IP 防刷：同一 IP 1 分钟内兑换请求超过 20 次，封禁该 IP 24 小时
- 迁移：`redemptions_per_user_activity_limit_v1`、`redemption_logs_user_id_idx_v1`（需重启后端）

### 涉及文件
- `backend/src/db/migrations.rs`、`models/redemption.rs`、`api/plugins/redemptions/mod.rs`、`middleware/rate_limit.rs`
- `frontend/src/pages/Redemptions/Redemptions.tsx`、`types/index.ts`

---

## 2026-08-02 — 控制台顶栏毛玻璃（用户端/管理端）

- `DashboardLayout` 顶栏改为半透明 + `backdrop-filter`；内容区铺满顶栏下方，滚动时穿过毛玻璃（用户端与管理端共用布局一并生效）

---

## 2026-08-02 — API 教程顶栏毛玻璃

- `/docs`（RelayAPI）顶部导航与模型广场一致：半透明 + `backdrop-filter` 模糊
- 修正布局：文档滚动区铺满顶栏下方，滚动时正文穿过顶栏，毛玻璃才可见（此前 `paddingTop` 在外层导致只能糊纯色底）

---

## 2026-08-02 — 用户端模型广场顶栏毛玻璃

- `/home/models` 顶部导航改为半透明 + `backdrop-filter` 模糊，内容滚动时可透过顶栏看到毛玻璃效果

---

## 2026-08-02 — 上游渠道配置启用/禁用状态

- `channel_configs` 新增 `status`（1=启用 / 0=禁用），管理端列表支持全部/激活/已禁用筛选与操作栏一键开关，编辑弹窗可改状态
- 模型渠道分组选择上游时，禁用配置置灰不可新选；选渠路由与 HA 子配过滤会跳过禁用上游，绑定后也无法实际使用
- 重启后端以执行迁移 `channel_configs_status_v1`

### 涉及文件
- `backend/src/db/migrations.rs`、`models/channel_config.rs`、`api/channel_configs.rs`、`relay/router.rs`
- `frontend/src/pages/Channels/ChannelConfigs.tsx`、`Channels.tsx`、`types/index.ts`

## 2026-07-31 — 系统概览：模型分布明细固定近 3 天

- 排行/进度条仍按当前筛选区间（如「今日」）；标签明细锚定区间末日向前固定 3 个自然日，不再被短区间截成一天
- 历史日一次读 `usage_daily_stats` 归档，仅当日走 realtime，避免按日循环放大查询

### 涉及文件
- `backend/src/api/date_helper.rs`、`dashboard.rs`
- `backend/src/relay/usage_stats.rs`
- `frontend/src/pages/Dashboard/Dashboard.tsx`

## 2026-07-31 — 出站 HTTP 客户端连接层加固

- 新增 `services/http_client`：建连超时、TCP keepalive、空闲连接回收；共享 Client 不设总超时（流式安全）
- 非流式上游请求统一挂 1800s 防挂死超时（可用环境变量覆盖）；火山监控 Client 复用同一连接基线
- README 补充可选 `HTTP_*` 调优环境变量

## 2026-07-31 — 数据同步：多站点请求密钥

- 本站密钥改为多密钥：可命名/备注、设置有效期、配置 IP 白名单（留空不限制）
- 导出鉴权匹配任一启用且未过期密钥，并校验 IP；旧单密钥自动迁移到 `data_sync_keys`

## 2026-07-31 — 站点插件「数据同步」

- 新增系统内置插件 `data_sync`：源站生成站点请求密钥，下游配置 URL + 密钥后可拉取模型目录与计费规则
- 冲突策略本站优先（同 mid/pid 跳过）；不同步折扣与 `group_ratios`；转发规则仅映射本站已有 eid/name，不创建；渠道不同步
- 管理端：本站密钥 / 拉取同步（测试连接、预览 Diff、确认同步）/ 同步日志；公开导出 `GET /api/v1/plugins/data_sync/export/bundle`

## 2026-07-30 — HA/选渠日志字段改中文描述

- `[HA]` / `[选渠]` / `[Chat]` 信息日志的 key 改为中文（状态码、上游YID、冷却等）；仅文案，逻辑与性能无实质影响

## 2026-07-30 — 上游失败状态码：日志 / HA / 客户端对齐

- `norm_status`：`record_zero_cost_fail` 落库与 `upstream_fail` 共用，避免日志写原码、对外被改成 502 的分叉；文案侧私有 `norm_err_msg`
- 火山 TTS 兼容路由：记账返回的推断状态码再构造 `upstream_fail`（原先日志推断、客户端常落 502）
- HA `first_fail` / reinstate / 熔断仍读同一 `UpstreamHttpError` 状态码；主路径 HTTP 非 2xx 行为不变

## 2026-07-30 — HA 熔断写入迁入 ha.rs

- `trigger_ha_meltdown` 从 `proxy.rs` 挪到 `ha.rs` 私有，与 `try_failover` / `scrub` 同模块；`proxy` 不再承载 HA 熔断
- 日志前缀统一 `[HA]`；判定统一 `is_ha_aid`；冷却表/白名单/周期 scrub 行为不变

## 2026-07-30 — 清理无调用启动种子化与误导配置

- 删除从未被 `main` 调用的 `Database::seed_admin`，以及仅为其服务的 `AppConfig.admin_username/password`
- 同步清理 `.env.example` / compose / `deploy.sh` / README 中的 `ADMIN_*`、`SEED_ADMIN_ON_BOOT`（管理员改由网页初始化页创建；运行时行为与改前一致）
- 去掉 hyperbc 多余 `#[allow(unused_imports)]`；全仓仍无 `#[cfg(test)]` / `#[test]` / `*.test.*`（保留产品页 ChannelTest）

## 2026-07-30 — relay 失败结算：规范锚点、暂缓再抽层

- `DEVELOPMENT_RULES.md` 补充零费用失败结算两入口 + `record_zero_cost_*` 选用表（行为零变更）
- 调用点样板已够用：再抽 builder / 合并判定会抬高回归风险；本仓不留存 `#[test]`，验证靠编译与手工路径

## 2026-07-29 — relay 零费用失败结算扩展复用

- `record_zero_cost_fail`：只记账，返回 `(status, client_msg)`，供 BadRequest / PaymentRequired 等非 `upstream_fail` 出口
- `ZeroCostUpstreamFail` 增加 `pre_deducted` / `pre_deduct_gift`、`upstream_req_content: Option`
- 已接入：image 同步轮询失败/超时/空图、video 素材转换失败记账、预扣费失败记账、audio 火山 TTS 错误记账
- **刻意未改**：成功计费、流结束结算、素材转换/预扣仍返回 BadRequest/PaymentRequired（避免变成 UpstreamHttpError 触发 HA）

## 2026-07-29 — relay 零费用上游失败结算收拢

- 新增 `proxy::record_zero_cost_upstream_fail` / `ZeroCostUpstreamFail`：两入口判定不变（HTTP 非 2xx vs body 业务失败），失败后统一记账 + `upstream_fail`
- 状态码：有 HTTP 失败码则优先用；否则从 body 推断。日志/客户端文案按原模块约定（含 chat 主路径短句、image/video post 的 format 体）
- 已接入 image / video / audio / generic / chat / native；流式成功结算与火山 TTS 专用路径未改

## 2026-07-29 — ha.rs 失败链收拢 + API 降可见性

- 删除薄包装 `on_spawn_fail` / `on_spawn_attempt_fail` / `on_attempt_fail`；上游失败逻辑并入 `HaAttempt::on_spawn_result_err`
- 仅内部使用的符号改为私有；对外保留 `HaAttempt` 与 `policy` / `yid_label` / `channel_is_ha_flag` / `resolve_log_config_id` / `is_melted_down` / `scrub_failed_channels`
- 行为不变：first_fail、400 系 skip-melt、业务侧 finish 优先

## 2026-07-29 — API 教程移除「火山素材库接口」菜单

- 默认文档树不再包含 `volcengine-assets-guide`；存量可在后台「恢复默认文档」清掉
- 不影响火山方舟 / MediaKit 教程及其他协议文档；素材库业务 API 本身未改动
- 精简 DocsApi：去掉增量种子/下线迁移与专用 seed 函数（Codex/Claude、级联指南、素材库），统一靠后台「恢复默认文档」对齐默认树；保留表结构/插件开关/slug/intl 迁移
- 复查：无 `#[cfg(test)]` / `#[test]` / `*.test.*` / `*.spec.*` 残留（保留产品页 ChannelTest）

## 2026-07-29 — 火山验证码变量名填写说明

- 管理端提示：控制台显示 `${1}` 则填 `1`，勿带 `${}`；按填写原样组 TemplateParam

## 2026-07-29 — 短信/验证码链路复查加固

- 火山国际号保留 E.164（不再误剥 `+`）；成功响应强制要求 MessageID
- 发送前校验消息组/SdkAppId 与签名；服务商切换文案改为「仅一套凭证」避免误解
- 验证码有效期统一为 `VERIFICATION_CODE_EXPIRY_MINUTES=5`（落库与邮件正文同源）
- 再扫：无单测残留；去掉仅内部转发的 `provider_kind`，HMAC 复用 `volcengine::hmac_sha256`

## 2026-07-29 — 短信通知支持火山引擎（原生 HTTP，无 SDK）

- `SmsSettings.provider`：`tencent`（默认，兼容旧配置）| `volcengine`
- 火山 SendSms：复用 `volcengine_sign`，消息组 ID 映射 `sdk_app_id`；验证码走 `TemplateParam` JSON（变量名 `code_param`，默认 `code`）；余额提醒无变量
- 腾讯云路径不变；管理端可切换服务商并分别配置凭证/模板/测试发送

## 2026-07-29 — 清理无用类型/导入与短信辅助拆分

- 全仓复查：仍无 `#[cfg(test)]` / `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）
- 删除未引用前端类型：`FinanceRechargeRecord`、`SMTPSettings`、`SmsSettings`、`GoogleOAuthSettings`、`WechatOAuthSettings`
- 去掉 EmailNotification / OAuthSettings 无用导入；`send_with_template` 改为私有
- `apiErrMsg` / `SKIP_ERR` 抽到 `utils/apiErr.ts`，短信提示组件只负责模板文案

## 2026-07-29 — 营销「提示通知」去掉重复短信模板文案

- 完整模板申请正文仅保留在「站点设置 → 消息通知 → 短信」
- 营销侧只保留开关、缺模板一行警告，以及指向短信配置的简短说明

## 2026-07-29 — 余额提醒短信改为无变量模板

- 腾讯云余额提醒模板不再传 `TemplateParamSet`（避免 `TemplateParamSetNotMatchApprovedTemplate`）
- 测试短信只传手机号；邮件测试仍可用余额/阈值预览变量
- 管理端文案与可复制申请正文同步为无变量固定模板

## 2026-07-29 — 清理无调用死代码（无单测残留）

- 全仓复查：无 `#[cfg(test)]` / `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）
- 删除未使用：`BillingIngress::enqueue`、`ApiToken::has_quota`、`BillingRule::apply_time_multiplier`、`Model::get_group_ratios`/`get_multiplier_for_group`、`tos::get_object_tags`

## 2026-07-29 — HA 终态与转发规则路径收紧

- `finish_err`：业务侧错误（余额/鉴权/BadRequest）优先于 first_fail，避免上游失败后被 402/400 盖住仍回上游文案
- image/video/audio/generic：转发规则不匹配改为 `on_access_err`+`break`（与 chat 一致，走 `ha.finish()`）
- 删除无调用的 `HaAttempt::reset_attempts`；熔断路径 yid 只格式化一次

## 2026-07-29 — 选渠/上游/ModelMeta 日志补回 info

- `[SelectChannel] start|db_candidates|after_filter`、`[Image→Upstream]`、`[ModelMeta]` 从 debug 升回 info（默认 `RUST_LOG=info` 可看）；preset / Image path=* 等仍 debug

## 2026-07-29 — chat_completions 去掉 EP 多模型空壳

- 删除恒为单元素的 `model_list` for、恒 `None` 的 `ep_tag` / `resolved_ep_models`；权限校验一次后直接进 HA while（与 responses 同构）

## 2026-07-29 — chat HA 失败路径与其它 relay 统一

- `chat_completions` 去掉平行的 `try_failover` / `trigger_ha_meltdown`；改为记账后 `HaAttempt::on_spawn_result_err`（与 image / responses 同构）
- 非首次失败由 `reinstate_first_log` 还原 first_fail；删除 `chat_upstream_fail` / `record_failed_billing`

## 2026-07-29 — HA 同档子渠按绑定序（不再随机）

- 子渠选中：`priority` 降序 → 同档 `weight` 降序 → 再同则按 `sub_channels` 绑定序；去掉加权随机，错误终态稳定
- 物理渠之间的优先级/权重选渠逻辑不变；failover / first_fail 语义不变

## 2026-07-29 — 本地默认 info；HA 关键日志升为 info

- `dev.sh` 等默认 `RUST_LOG=info`；需要明细时 `RUST_LOG=debug ./dev.sh`
- HA begin/switch/stop/finish/熔断/skip-melt 与选渠 `picked` 改为 `info`（无级别分支；异常 reinstate/软上限仍 `warn`）

## 2026-07-29 — HA/选渠日志补 yid

- `[SelectChannel] picked`、`[HA] switch/stop`、熔断/白名单日志增加 `yid=`，便于对照后台上游 YID（原先只有 config id / aid）

## 2026-07-29 — 本地开发默认 RUST_LOG=debug

- （已由同日「默认 info」条目取代）原将 `dev.sh` 等默认改为 `RUST_LOG=debug`；现改回默认 `info`，debug 需显式开启

## 2026-07-28 — 删除 TRUSTED_PROXIES（整项废弃）

- 去掉环境变量 / `AppConfig` / compose / `.env.example` / README 中全部 `TRUSTED_PROXIES` 说明，避免误配
- `extract_client_ip`：优先 `X-Forwarded-For`（首段）/ `X-Real-IP`，否则 socket IP；与 Docker 反代取真实 IP 行为一致

## 2026-07-28 — Mac 导出逻辑精简与加速

- `export-images.sh` / `push-images.sh` 内联 zigbuild + PREBUILT（不抽公共脚本文件）
- 预编译镜像阶段去掉 `apt install file`；`USE_PREBUILT=1` 时 frontend+backend 并行
- 仓库在 `/Volumes` 时 zigbuild 的 `CARGO_TARGET_DIR` 迁到本机 SSD 缓存；开关与用法不变

## 2026-07-28 — zigbuild 交叉编译启用 vendored OpenSSL

- 根因：`cargo zigbuild` 到 `*-linux-gnu` 时 `openssl-sys` 找不到 Linux OpenSSL sysroot 而失败，无法生成 `tokensbyte-server-bin`
- 新增可选 feature `cross_compile`（`openssl` vendored）；导出/推送脚本的 zigbuild 自动带上
- 日常与 Docker 内编译默认不启，依赖面不变

## 2026-07-28 — 计费入口去掉重复 round

- `check_access` 直接用 `ctx.balance`（`get_user_context` 已 round）；注释微收紧

## 2026-07-28 — 结算差额口径抽到 money::settlement_delta

- 同步 `record_and_bill_inner` 与异步 `execute_settlement_tx` 共用 `settlement_delta`；预扣金额读库后 `round_money`
- 行为不变：应付原样落账、赠送优先、余额可负

## 2026-07-28 — 修复计费 cost=0 与空钱包仍可调用

- 入口：`check_access` 对有/无限额令牌统一校验可用额（系统+赠送+信控，`round_money`）；预扣=0 且可用≤0 → 402
- 结算：同步/异步去掉授信封顶下调 `settled_cost`；应付金额原样写入 `logs.cost`，赠送优先后 `balance` 可扣成负
- 删除无用 `cap_additional_charge`；避免「有计费依据却 cost=0」导致平台亏费

## 2026-07-28 — 素材转换判定微精简

- `is_base64_media`：去掉与「含 `:` 即非纯 base64」重复的 http/asset 分支；判定结果不变
- 复用已够（共用解码/TOS/Create）；按规范不新增留存单测，不做更大抽层

## 2026-07-28 — 素材转换支持纯 base64

- 插件/上游素材转换：`content` 中除 `data:` URI 外，亦接受无前缀纯 base64（判定与 `forward::parse_image_data` 对齐）
- 共用 `is_base64_media` + `decode_base64_data`（魔数推扩展名，未知则按 Image/Video/Audio 默认）；http(s) 路径不变

## 2026-07-28 — 清理无用代码与 scratch 脚本

- 全仓确认无 `#[cfg(test)]` / `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）
- `PluginConfig`：去掉未用 `ApartmentOutlined`、死别名 `TOS_REGIONS`；`INDEPENDENT_STORAGE_PLUGINS` 提升为模块常量
- 删除本地 scratch `scratch_restore.sh`，并去掉 opensource 打包对其排除项

## 2026-07-28 — 素材转换/存储配置微提炼

- `asset_convert`：成功写回复用 `push_convert_ok`；行为与日志格式不变
- `PluginConfig`：独立存储插件名单收敛为 `INDEPENDENT_STORAGE_PLUGINS`（含 upstream_asset_relay）
- 按仓库精简规范：不新增留存单测

## 2026-07-28 — 上游素材插件存储改为可单独配置

- 「火山视频转素材ID」存储配置改为与素材中心同类的独立 TOS 表单（可保存/测试连接）
- 未单独配置时运行时仍回退「站点设置 → 存储设置」全局 TOS，不影响已有 base64 转换

## 2026-07-28 — 上游素材插件增加全局存储配置页

- 「火山视频转素材ID」插件配置增加「存储配置」页签（后续已改为可单独配置，见上条）
- 便于确认 base64→TOS 临时上传所用存储来源；运行时仍走原有 `get_tos_config` 回退，不影响转换逻辑

## 2026-07-28 — 上游素材转换支持 base64（与插件路径统一）

- `upstream_asset_convert`：base64 data URI 与 `asset_convert` 共用「解码 → TOS 临时 URL → CreateAsset(URL) → asset:// → 清理临时对象」
- 抽取 `convert_base64_with_create` / `create_asset_via_upstream`，去掉上游「不支持 base64」分支；需系统或插件 TOS

## 2026-07-28 — Mac 导出强制 zigbuild，杜绝 Docker cargo OOM

- 根因：zigbuild 失败/跳过后仍继续容器内 `cargo build --release`，易 SIGKILL / cannot allocate memory
- `export-images.sh` / `push-images.sh`：Mac 必须先产出匹配架构 Linux ELF；`docker compose build --build-arg USE_PREBUILT=…`；仅 `ALLOW_DOCKER_CARGO=1` 才允许容器内编译
- `backend/Dockerfile`：构建阶段显式 `ARG USE_PREBUILT` 选择预编译路径；README / Mac 指南同步

## 2026-07-29 — format_as_openai_error 去重

- 门禁只留 `is_upstream_error_response`；去掉重复的 `/error/code` 手写解析与二次读 `error.message`

## 2026-07-29 — format_openai 错误分支去重

- 去掉外层重复的 `is_upstream_error_response`，仅经 `format_as_openai_error` 一次门禁

## 2026-07-29 — 上游错误 OpenAI 规范化收口

- `format_as_openai_error`：厂商错误转换 + 脏 OpenAI（`success`/数字 type）一并规范化
- `normalize_upstream_error_for_client` 缩为 parse → formatter，去掉重复 ErrorCode/error.message 分支

## 2026-07-29 — HA 切换停环修复与错误 OpenAI 格式

- 上游 400/403/422：HA **仍切换**下一子渠，仅跳过熔断（原逻辑直接停环导致绑 3 个只打 2 个）
- 上游错误对外统一收成标准 OpenAI `error{message,type,code}`，去掉网关 `success` 等字段
- 精简 HA 调试日志：`[HA] begin/switch/stop/finish`；SelectChannel/Image/ModelMeta 降为 debug

## 2026-07-28 — 修复 logs.is_ha 解码类型

- `is_ha` 列为 `INTEGER`（INT4），模型字段由 `i16` 改为 `i32`，消除 sqlx 解码报错

## 2026-07-28 — 日志 HA 标志改为写时快照

- `logs.is_ha`：请求当时是否走高可用组；预记录/结算/渠道测试写入，读列表用快照不再 JOIN `channels.provider_type`
- 避免事后把普通渠道改成 HA 组后，历史非 HA 日志被误标；存量日志默认 `is_ha=0`（保守不误导）
- `logs` / `logs_archive` 各 `ADD COLUMN is_ha`；归档改为 jsonb 按列名写入，不再依赖 `SELECT l.*, NOW()` 列序
- `logs_archive_v1` 主键幂等改为 `EXCEPTION WHEN duplicate_object`（等价更短；已执行环境仍跳过该 ID）

## 2026-07-28 — Mac 导出镜像自动 cargo-zigbuild 预编译

- `export-images.sh` / `push-images.sh`：Darwin 选定目标 arch 后自动 `cargo zigbuild` → `tokensbyte-server-bin` → `USE_PREBUILT=1`（后续已改为失败即停，见上条）
- README 与 `docs/Mac开发环境打包与交叉编译指南.md` 同步

## 2026-07-28 — 移除未使用 TipTap 与直接 openssl 依赖

- 前端：删除未被引用的 `TipTapEditor` 及全部 `@tiptap/*` 依赖、相关 CSS；富文本仍用 `react-quill-new` / `md-editor-rt`
- 后端：去掉 `Cargo.toml` 中无业务引用的直接 `openssl`（vendored）；Linux 上仍可由 `reqwest`→`native-tls` 传递引入系统 OpenSSL，不影响 TLS
- 清空 knip 对 TipTap 的 ignore；功能路径不变

## 2026-07-28 — 合并 cgdev0726 门户与 DOCS 增强

- 合并 `origin/cgdev0726`：保留赛博门户 GitHub 入口、游客 PluginRoute、site_portal_pro DOCS/模板与风格切换等改动
- 保留本分支 Chat/Anthropic 文档拆分与用户端日志脱敏；合并时去掉对方带入的 `#[cfg(test)]`（含 site_icons / site_portal_pro）
- 文档页样式类：门户用 `cyber-hacker-docs`，API 教程用 `docs-api-system`

## 2026-07-28 — 用户端日志脱敏与响应精简

- 普通用户 `/logs`、`/logs/{id}/detail`、`/task_logs`、仪表盘最近活动：不返回上游出参；计费明细去掉渠道/模型映射；清空 PID/EID/YID；`plugin_tag` 仅保留 `client_ct`/`title`
- 抽取 `redact_request_log_for_user` / `user_allow_view_log_details` 等复用；详情级联脱敏仍读库原 `plugin_tag`，不再折叠已隐藏的上游出参
- 超管后台与库内完整字段不变；全仓无 `#[test]` / `*.test.*` 残留（保留产品页 ChannelTest）

## 2026-07-28 — 日志错误码下拉补 422 并多语言

- 常用错误码选项增加上游常见 `422`；文案迁入 zh/en/ja/ko/vi 的 `logs.status_code_*`
- 占位符 `logs.search_status_code` 同步五语；选项构建去掉多余 `useMemo`（14 项直接 map）

---

## 2026-07-28 — 钱包余额不足统一返回/落库 402

- 新增 `AppError::PaymentRequired` → HTTP 402
- `check_access` / `pre_deduct_or_intercept` 余额不足由 403 Forbidden 改为 402；HA 仍禁止 failover，且 402 不触发渠道熔断
- 抽出 `ha::is_client_side_http_status`（400/402/403/422）供 failover / 熔断共用，去掉重复字面量
- Playground 持久化重试将 402 视为业务永久错误（与原 403 余额不足行为对齐）
- 令牌额度耗尽、模型未授权等权限类错误仍为 403，行为不变

---

## 2026-07-28 — 今日改动复查修补

- 日志错误码 Select 清空时归一为 `undefined`，避免 antd 传入 `null`
- 主题推广有效期：前端改传 ISO（含时区）；后端无偏移墙钟按站点默认 timedisplay 解析，修正原「按 UTC 解读」偏差
- 主题推广点击统计 key / 上报 slug 统一小写，与入库及团队邀请展示对齐
- 删除合并带入的 `#[cfg(test)]`（auth / theme_promo / link_click）；全仓复查仍无 `#[test]` / `*.test.*` / `*.spec.*`（保留产品页 ChannelTest）

---

## 2026-07-28 — 日志错误码改为常用码下拉

- 筛选栏「错误码」由数字输入改为常用状态码 Select（含 0/4xx/5xx），可搜索、可清空
- 仍精确匹配 `status_code`；查询/导出/重置行为不变

---

## 2026-07-28 — 合并 cgdev0714time 主题推广与邀请追踪

- 并入主题推广落地页、营销链接点击统计、邀请 1 天锁定与 OAuth 归因、`dev.sh` Mac 编译优化
- 保留本分支日志错误码筛选、API 文档 Chat/Anthropic 拆分、方舟监控主题适配；团队选人仍走 `/users?keyword=`（不恢复已删的 `search-users`）

---

## 2026-07-28 — 方舟监控凭证/绑定页去掉白底卡片

- 主账号凭证、Endpoint 绑定去掉多余 Card 白底，与页面背景融为一体（白天主题不再套白块）

---

## 2026-07-28 — 方舟监控绑定/账号页主题适配

- Endpoint 绑定、主账号页去掉硬编码浅色字色与主按钮渐变，改用 `theme.useToken`
- 次要信息用 `colorTextTertiary`；等宽 ID/密钥用 `fontFamilyCode` 的 span，避免暗色主题对比失效

---

## 2026-07-28 — 清理调试日志与未用导入

- 去掉 WalletDetailsView 每次渲染的 debug `console.log`
- PluginsList 插件列表拉取失败改为静默兜底（功能不变）
- Logs 页移除未使用的 antd/lucide/`CardActions` 导入
- 全仓确认无 `#[test]` / `*.test.*` / `*.spec.*` 残留（保留产品页 ChannelTest）

---

## 2026-07-28 — 日志筛选参数构建去重

- `buildLogListParams` 统一列表/导出拼参；`pickFilter` 正确处理重置时显式 `undefined` 覆盖
- 精简 `parseStatusCodeFilter`；临时校验后无测试残留

---

## 2026-07-28 — 日志页支持错误码精确筛选

- `LogQuery` 新增 `status_code`，WHERE 精确匹配 `logs.status_code`；与原有 success/fail 互不干扰，导出共用同一条件
- 日志筛选栏增加「错误码」输入（仅数字），查询/回车/导出均生效；重置一并清空

---

## 2026-07-27 — Anthropic 聊天文档仅保留 base64 图片

- 确认网关 Anthropic 透传不转换图片；文档/PDF 无接入处理 → 不写
- 去掉图片 URL 示例，多模态仅保留 base64（含多图）；与线上可用方式对齐

---

## 2026-07-27 — 聊天文档去掉无效可选字段

- 删除 `image_url.detail`（网关未处理；协议转换只取 `url`）
- Anthropic 文档去掉未接入的 `document`/PDF 说明，参数表仅保留实际用到的字段

---

## 2026-07-27 — 常用示例拆分聊天 Chat / Anthropic

- 「常用调用示例」原合并篇拆为 `聊天 Chat`（OpenAI 兼容）与 `聊天 Anthropic`（原生 Messages）
- Chat：纯文本 / 图片 / 音频 / 视频 curl；Anthropic：纯文本 / URL·Base64 图片 / 多图 / PDF 简介
- 删除旧 `claude-chat` 种子；docs_api 与 site_portal_pro 同步

---

## 2026-07-27 — 图像 API 默认文档去掉 SDK tabs

- docs_api / site_portal_pro 默认文档中图像示例（gpt/volc/kling/google）去掉 Python/Node tabs，仅保留 curl
- `common-errors` 错误类型 tabs 保留；zh 图像编辑仍保留 JSON + Form-Data curl

---

## 2026-07-27 — 精简聊天示例并补齐 Anthropic 多模态

- 「常用调用示例 / 聊天对话」去掉冗长 SDK tabs 与响应样例，改为 curl 纯文本 + 多模态
- OpenAI 与 Anthropic 原生均补充单图 + 文本；Anthropic 注明 URL / base64 `source` 写法

---

## 2026-07-27 — API 教程聊天示例补充多模态单图

- 「常用调用示例 / 聊天对话」增加 OpenAI 兼容单图 + 文本（URL）curl 示例；参数表注明 `content` 可为多模态数组

---

## 2026-07-27 — 今日模型搜索改动复查修补

- `/models` 的 `page_size`：忽略 ≤0、上限 10000，避免异常 LIMIT
- Models 列表写入用 `Array.isArray(data)` 兜底，防止畸形响应导致渲染崩溃
- ModelSelector 类型补 `model_id_alias`，与共用关键词匹配字段对齐

---

## 2026-07-27 — 清理死状态与确认无测试残留

- 全仓确认无 `#[cfg(test)]` / `#[test]` / 前端 `*.test.*` 残留（保留产品页 ChannelTest）
- Models 删除从未读写的 `classStats` 及无用类型导入

---

## 2026-07-27 — 解耦 API 教程与站点门户增强版文档数据源

- `/docs`（docs_api）不再优先请求 `site-portal-pro`；空树也不再误占位导致「暂无文档」
- 路由显式绑定：`/docs` → docs-api，`/home-pro/docs` → site-portal-pro；两套库表与开关互不影响
- 共享 `RelayAPI` 仅按传入 `apiPrefix` 取数，去掉跨插件 try/catch 回退

---

## 2026-07-27 — 模型筛选参数/绑定再提炼

- 抽出 `buildClassificationParams`，Models / PluginConfig 共用
- `list_models` 筛选绑定抽 `model_filter_ids`，与 WHERE 顺序一致、去重 list/count 绑定
- 模型广场本地搜复用 `modelMatchesKeyword`（保留 original_id / provider / description / variants）
- 本链路无单元测试残留；不硬抽 stats 三段 SQL（JOIN 交叉条件易改坏角标）

---

## 2026-07-27 — 模型搜索链路复查修补

- `/models` 无 page_size 时用列表长度作 total，去掉多余 COUNT
- Models 分类请求加 QueryGuard，避免快切筛选时旧响应覆盖新数据
- 抽出 `modelMatchesKeyword` 统一管理端模型本地搜字段；PluginConfig 分类判断改 `!= null`；团队选人 keyword trim

---

## 2026-07-27 — 同类搜索冗余一并精简

- 删除废弃 `Channels.tsx.patch`
- 团队营销选人改为复用 `/users?keyword=`，移除重复的 `/team-marketing/search-users`
- 插件配置 / ModelSelector 本地模型搜对齐备注字段；stats/models 服务端 search 已在此前去掉

---

## 2026-07-27 — 去掉模型列表/分类统计的服务端 search

- 管理端列表已本地搜，删除 `/models` 与 `/classifications/stats` 的 `search` 参数及相关 SQL
- 插件配置页 stats 不再跟搜索关键词；列表仍本地筛，分类变化才刷角标

---

## 2026-07-27 — 模型管理搜索失效修复（备注 / 旧后端）

- 列表 `/models` 按分类拉取，关键词（含备注）本地筛；打字不打 API

---

## 2026-07-27 — 模型管理搜索归 /models，stats 不再跟 search

- 列表与 stats 职责拆分

---

## 2026-07-27 — 修复模型管理备注搜索不生效

- 列表关键词本地筛选（名称 / Model ID / MID / 别名 / 备注）

---

## 2026-07-27 — 清理单元测试与仅测用代码

- 移除 backend 全部 `#[cfg(test)]` 测试模块（forward / live_metrics / runtime_info / site_portal / site_portal_pro）
- 删除 site_portal_pro 中已标注「仅单测保留」的托管首页辅助函数；生产路径（自定义主页 / 风格首页）不变
- 保留产品功能页（如通道测试 ChannelTest）与浏览器测试 skill

---

## 2026-07-27 — 模型管理列表支持按备注搜索

- 管理端模型列表与分类统计的 `search` 增加 `remark ILIKE`；搜索条件抽成共用片段，列表/COUNT 共用 WHERE，避免多处漂移
- 搜索框占位补充「备注」

---

## 2026-07-27 — 级联进行中：用户端日志/任务列表掩码阶段一成功结果

- 普通用户端：级联未完成时详情响应强制为处理中形态，并硬保证不含 http(s) 产物地址
- 仅在 `plugin_tag` 含 cascade 时修补目标分辨率（避免无关 plugin_tag 被默认成 720p）
- 任务列表对未完成任务不返回预览 URL；管理员与完成后行为不变

## 2026-07-28 — 赛博顶栏操作区尺寸对齐

- 语言/矩阵圆钮统一 36×36，登录/注册同高同圆角；双语占位改为同行叠字，避免 Login 被撑高

---

## 2026-07-28 — 赛博语言按钮显示当前语种

- 顶栏语言钮改为显示当前语言（中文页「中」、英文页「EN」），不再显示「即将切换到」的语种

---

## 2026-07-28 — 赛博登录/注册按钮中英等宽

- 默认文案改为 `登录|Login`、`注册|Sign Up`（去掉过长的 Get API Key）；旧配置自动迁移
- 顶栏与 hero 按钮用中英双文案占位，切换语言时宽度不跳动

---

## 2026-07-28 — 赛博门户首页 Hero 文案改写

- 主标题改为「大模型 API 中转开源系统，一处接入全球模型」；副标题突出 OpenAI 兼容、密钥/限流/路由/计费与可自托管

---

## 2026-07-28 — 赛博顶栏喇叭改为中英语言切换

- 右上角原音效按钮改为 `中` / `EN` 切换，写入 `localStorage.lang`，与门户双语 `|` 文案一致
- 首页硬编码区块补齐中英 `data-i18n-dynamic`；矩阵雨保留，音效开关移除

---

## 2026-07-28 — 赛博子页顶栏与首页共用同一组件

- 抽出 `_chrome_header` / styles / scripts；首页与模型广场等子页共用同一套顶栏 HTML（含音效/矩阵按钮与 CRT 底纹），不再用 CSS 近似

---

## 2026-07-28 — 门户模型广场顶栏对齐首页

- 赛博黑客风格下，模型广场等子页顶栏与首页一致：Orbitron 品牌、脉冲方标、`//` 导航、霓虹登录/注册按钮

---

## 2026-07-28 — 门户模型图标统一解析与回退

- 模型列表/详情图标统一：模型 logo → 服务商 → 类型 → lobe 默认图；裸名自动拼 `/assets/icons/lobe/*.svg`
- 加载失败降级到默认图，再失败显示字母占位，避免裂图或不显示

---

## 2026-07-28 — 门户模型页 UI 优化与死链清理

- `/home-pro/models` 列表页重排：粘性筛选、结果计数、移动端筛选按钮，并适配 cyber_hacker / tech / dark_gradient
- Docker/Nginx 不再把门户模型页打到 SPA；底部默认链接改为真实门户路径，空/#/旧锚点死链不渲染

---

## 2026-07-28 — 门户管理配置对各风格页生效

- 共享 `base.html` 底部改为读取门户管理扁平 `footer_config`（品牌/链接/资讯/法律信息），经典/科技/星空/赛博子页均生效
- 赛博黑客首页链接加 `| safe`，避免 `/` 被 HTML 转义导致登录注册等配置链接失效
- 补充各风格首页与关于页渲染单测，校验导航/底部/SEO/脚本注入

---

## 2026-07-28 — 门户导航模型广场改走 /home-pro/models

- 默认「模型广场」改为门户页 `/home-pro/models`（非站点插件 `/home/models`）
- 新增动态路由渲染门户模型列表；已保存 `marketplace` 菜单若仍指向 `/home/models` 会自动迁移

---

## 2026-07-28 — 门户导航管理增加预览按钮

- 顶部导航菜单每项在删除按钮前增加预览，新窗口打开对应链接（锚点拼到首页）

---

## 2026-07-28 — 门户默认导航精简为文档与模型广场

- 默认顶部导航移除「平台优势 / 核心功能 / 模型矩阵 / 接入指南」；仅保留「文档」「模型广场」
- 已保存配置中的上述首页锚点菜单加载时自动剔除

---

## 2026-07-28 — 门户默认导航增加 DOCS

- 顶部导航默认项新增「文档|Docs」→ `/home-pro/docs`；已保存配置若缺少该项会自动补上

---

## 2026-07-28 — 赛博黑客门户首页接入门户管理配置

- 赛博黑客风格首页读取门户管理中的导航 / 底部 / SEO / 自定义脚本（统计与客服），保存后生效到 `/home-pro`
- 仅首页走赛博整页模板；关于/联系等子页仍用共享 chrome，避免丢失配置导航

---

## 2026-07-27 — 用户端 DOCS 切换分类默认打开首篇

- 站点门户用户端 docs：点击顶部分类后自动打开该分类下第一篇文章；无 URL 时默认分类亦打开首篇

---

## 2026-07-27 — 渠道指南路由示意图排版修复

- 「渠道管理与负载均衡」文档去掉易乱码的 ASCII 框图，改为表格 + 层级列表说明优先级/权重调度

---

## 2026-07-27 — 站点门户 DOCS 分类可排序与设默认

- 分类管理弹窗支持上移/下移调整顺序、「设为默认」、编辑名称与排序值、增删分类
- 前后台分类列表按 `sort_order` 排序（默认分类不再强制置顶）

---

## 2026-07-27 — DocsApi API教程页去绿色强调色

- 用户端 `/docs` 翠绿强调改为 zinc 中性色，与侧栏/顶栏整体风格一致（分类选中、侧栏高亮、行内代码、引用、标题竖条等）

---

## 2026-07-27 — API文档标题去空格

- DocsApi / 站点门户 DOCS：中文种子标题去掉全部空格；用户端侧栏展示同步去空格（含已有库数据）

---

## 2026-07-27 — DocsApi API教程页顶栏对齐控制台

- 用户端 `/docs` API教程页右上角改为与登录后控制台一致：模型广场、API教程、主题切换、语言、通知、用户头像

---

## 2026-07-28 — 站点门户 DOCS 初始化数据对齐当前分类

- 「使用指南」初始化写入模型/渠道根级文章；「商务合作」写入商务合作文
- 「API 参考」目录树去掉模型/渠道（改归使用指南）；快速开始仅保留鉴权与端点

---

## 2026-07-28 — 站点门户 DOCS 品牌对齐赛博黑客主页

- DOCS 左上角品牌改为 Orbitron + 霓虹发光字 + 脉冲方标，与赛博黑客门户主页一致

---

## 2026-07-28 — 站点门户增强 DOCS 多语言

- 管理后台 DOCS：补齐语言页签、各语言标题/正文编辑、AI 一键翻译
- 用户端 `/home-pro/docs`：顶部右上角增加语言切换；树与正文按所选语言展示

---

## 2026-07-28 — 站点门户增强 DOCS 管理端改 shadcn 灰阶

- 管理后台「DOCS文档」去掉用户端赛博绿主题，改用与 DocsApi 一致的黑白灰 shadcn UI

---

## 2026-07-27 — 站点门户 DOCS「API参考」标题去空格

- 已并入「API文档标题去空格」：全部类别侧栏/种子标题去空格

---

## 2026-07-27 — 站点门户 DOCS 代码块配色分层

- 代码井改为更深冷青绿面板 + 薄荷字色，与正文霓虹绿区分；保留单层边框与顶栏复制区

---

## 2026-07-27 — 站点门户 DOCS 代码块去外边框

- 用户端 DOCS 正文代码区去掉双重外框：保留代码区本身单层边框，去掉再套一层大外边框

---

## 2026-07-27 — 站点门户 DOCS 手机端布局优化

- 用户端 DOCS：窄屏侧栏改为浮层抽屉 + 遮罩；默认收起；点文档自动关闭
- 顶部导航：手机端安全区适配、分类横滑、略加深透明毛玻璃以保证可读性
- 正文：缩小边距与标题字号；表格/代码块横向滚动；修复 TOC 抽屉无法打开
- 断点切换：PC↔手机同步收展侧栏（无「展开再关闭」闪动）；回到 PC 默认展开菜单

---

## 2026-07-27 — 站点门户 DOCS 顶部导航透明毛玻璃

- 用户端门户预览 DOCS 页顶部导航恢复透明毛玻璃：排除实色 `bg-[#…]` 覆盖规则，半透明底 + `backdrop-filter` 模糊

---

## 2026-07-27 — 安全加固（鉴权 / CORS / XSS 面）

- Site Icons：管理接口强制 DB 校验 admin；图标名白名单；写入路径禁止 `..`；SVG 拒绝 script/事件属性
- 登录：`redirect` 仅允许站内相对路径；OAuth 改为一次性 `code` 兑换 JWT；代入登录用 localStorage handoff，避免 JWT 进 URL
- CORS：非 `APP_ENV=development|dev` 且未设置 `CORS_ORIGINS` 时拒绝跨域；开发环境仍可 permissive
- 主题推广 iframe：去掉 `allow-same-origin`，防止营销 HTML 读取父页 `localStorage`
- 登录/管理登录按 IP 每分钟 10 次限流

---

## 2026-07-27 — 高级营销：推广链接点击统计

- 专属邀请 / 团队邀请 / 主题推广链接前展示累计点击次数
- 统计规则：同一 IP 在站点时区自然日内对同一链接只计 1 次；已登录推广员访问自己的链接不计次
- 主题页失效回退注册页时只记主题链接点击，避免重复计入邀请链接

---

## 2026-07-27 — 推广邀请 1 天锁定不被覆盖

- 首次通过邀请链接写入 `aff`/`team` 后，**1 天内**不接受其他推广链接覆盖；**3 天内**注册仍可归因
- 锁定期内即使打开新的邀请 URL，注册/OAuth 仍使用已锁定的业务员

---

## 2026-07-27 — 邀请参数统一捕获与 OAuth 归因

- 统一 `aff`/`team` 捕获与读取（localStorage + cookie，3 天 TTL），覆盖 `/register`、`/promo/{slug}`、登录页「去注册」、表单注册
- 微信/谷歌 OAuth 自动注册：邀请参数写入 HMAC `state`（query/cookie 发起时嵌入），回调优先读 state、cookie 兜底，写入 `referred_by`、邀请奖励并自动入团
- 主题推广 HTML 内 `/register` 链接自动补上当前邀请参数

---

## 2026-07-27 — 优化 Mac 本地开发编译（dev.sh）

- 仓库在 `/Volumes` 外置盘时，自动将 `CARGO_TARGET_DIR` 迁到本机 `~/Library/Caches/tokensbyte-dev/target/`
- macOS 默认自动 `brew install sccache`（不自动装巨型 `llvm`）；若本机已有 `ld64.lld` 仍启用链接加速
- `[profile.dev]` 恢复 `incremental = true`，加速 cargo-watch 热重载

---

## 2026-07-27 — 高级营销：主题推广落地页

- 站点插件「团队营销管理」新增「主题推广」Tab：可粘贴 HTML 单页、设置上线/下线与长期或时段有效期
- 公开地址 `/promo/{slug}?aff=`；活动无效时自动回退普通邀请注册页
- 用户端「高级营销」在专属邀请链接下展示可复制的主题推广链接
- `theme_promotions.status` / `is_permanent` 升为 BIGINT，与项目 i64 约定对齐

---

## 2026-07-27 — 保护模型分类自定义排序不被升级覆盖

- 迁移回填「视频增强」logo/remark 时不再强制改写 `sort_order`；官方服务商 / API 服务商 / 类型的排序由管理端配置，升级须保留
- 迁移规范补充：种子与回填禁止顺带 SET 上述三表的 `sort_order`

---

## 2026-07-27 — 火山视频转素材ID：添加规则按关联名命名

- 「添加规则」生成的转发规则名固定为 `{关联名}·转素材#{绑定ID}`（如 `移动·转素材#3`），便于辨认归属且无需冲突探测；转换逻辑不变

---

## 2026-07-27 — 高可用最大备用切换次数上限调整为 100

- 插件「最大备用切换次数」UI/API 上限由 10 调整为 100；保存后仍即时加载到内存，逻辑不变

---

## 2026-07-27 — 修复用户端「视频监控」菜单默认不显示

- 用户菜单项 `/ark-video-monitor` 默认改为启用；插件开启后自动出现在侧栏（仍受插件开关与菜单配置约束）

---

## 2026-07-27 — 修复方舟监控 cron 因 is_enabled 类型崩溃

- `CronArkMonitor` 读取 `plugins.is_enabled` 由 `i32` 改为 `i64`（列已是 BIGINT），消除 INT4/INT8 解码失败导致同步中断

---

## 2026-07-27 — 级联转发规则支持 480p 目标

- 级联目标分辨率新增 `480p`：阶段一 480p 生成，阶段二 480p 画质增强（同分辨率增强，非超分）
- 转发规则可配 `480p` 倍率 / 增强 / 场景；底座锁定 `480p`（与 720p 档一致，UI 禁用切换）

---

## 2026-07-27 — 火山方舟视频监控改为钱包熔断

- 删除绑定表独立 `limit_quota`；同步时按 `used_quota - wallet_charged_quota` 增量扣用户钱包（赠送金优先，现金可扣成负数），写 `ark_video_consume` / `ark_video_refund` 流水
- 可用余额 `balance + gift_balance + credit_limit <= 0` 时停用该用户全部绑定并 `StopEndpoint`；余额恢复后 cron 仅自动 `StartEndpoint` 钱包熔断项
- 绑定表 `fuse_reason`：`wallet`=余额熔断可自动恢复，`manual`=管理员停用（cron 不拉起），启用时清空
- 上线后首次同步对历史 `used_quota` 追扣；管理端启用绑定需钱包可用余额 > 0；看板展示钱包可用与已用
- 同步按接入点批量 `SUM`；汇总查询失败则跳过该账号入账（避免失败被当成 0 误退款）；待恢复用户钱包批量读取

---

## 2026-07-25 — 本地 dev 默认开启 debug 日志

- `dev.sh` / `dev.ps1`（及 `dev-os.sh` / `dev-all.sh` / `local_restart.sh`）默认 `RUST_LOG=info`；`RUST_LOG=debug` 可覆盖。部署/`docker-compose` 默认仍为 `info`（见 2026-07-29 条目）

---

## 2026-07-25 — 支付回调敏感日志降级

- 通联回调原始 body、HyperBC 验签明细（公钥前缀/签名/待签串）由 `info` 改为 `debug`，与微信/支付宝一致；验签与入账逻辑不变

---

## 2026-07-25 — DeepSeek 缓存命中 usage 兜底

- `usage_extractor`：根级无 `cached_tokens` 时，将 `prompt_cache_hit_tokens` 映射为 `cached`（与 Claude `cache_read_input_tokens` 同级兜底）
- 不新增字段；`prompt_cache_miss_tokens` 仍由 `prompt - hit` 体现。需配置读缓存费率后折扣才生效，未配置行为不变

---

## 2026-07-25 — 阶梯计费文案精简

- 管理端说明、占位符、计费明细、模型广场改为更短的运营向文案（入/出/读/写）；计费逻辑不变
- 阶梯计费 / 豆包聊天阶梯说明进一步精简，并保留费用公式

---

## 2026-07-25 — GPT官方计费展示更名为 GPT图片计费

- 仅改 UI/日志文案与市场文案；`billing_rule=gpt_billing` 不变，已有规则与计费逻辑不受影响

---

## 2026-07-25 — 阶梯计费兼容 GPT 缓存写入

- `UsageTokens` 增加 `cache_write`；Chat (`prompt_tokens_details`) / Responses (`input_tokens_details`) 双路径提取 `cache_write_tokens`，不混入 Claude `cache_creation`
- `tiered` 阶梯每档可选 `cache_write_rate`：未填时行为与旧版一致（写入量并入未缓存输入）；填写后按写入价独立结算
- 管理端阶梯计费 UI / RateDisplay 展示「写缓存」选填列；保存时规范化 `cache_write_rate`
- 结算拆分逻辑去重为 `split_prompt_subsets`；提炼 `enrich_features_from_usage` 并在 Chat/Responses/原生/日志快照路径统一挂载 `cache_creation` 与 `web_search`
- 验证用临时单测已删除，不留测试残留

---

## 2026-07-24 — 图片上游 Content-Type 透传

- `image.rs`：去掉「gpt + 含图片」强制改 multipart；客户端用 `application/json` / `multipart/form-data` 则原样请求上游
- 提交类型写入 `logs.plugin_tag.client_ct`（预记录即落库；即梦轮询覆盖时保留），后台使用日志列表/详情可展示
- 提炼 `content_type_is_multipart` / `client_content_type` / `plugin_tag_*` 纯函数复用；前端日志页复用 `parsePluginTagMeta`（出参区不再重复打相同 Tag）
- tracing 仅记 `client_ct`（透传无第二套值）
- 删除仅服务旧强制逻辑的 `has_image_inputs`；验证用临时单测已移除，不留测试残留

---

## 2026-07-24 — 精简确认未用死代码（不影响功能）

- 删除未挂载的 `monitor/health.rs`、未引用的 `tos::download_file`、`VolcClient` 未调用的 `sign`/`hmac_sha256` 包装
- 删除未注册的 `examples/hash_pass.rs`；`auth` 改用已有 `hex` crate，去掉本地重复 `mod hex`

---

## 2026-07-24 — lettre 再收 feature（不影响发信）

- `tokio1-rustls-tls` → `rustls-tls`：业务只用同步 `SmtpTransport`，不再拉 lettre 的 tokio async SMTP 栈
- 仍保留 `builder` / `smtp-transport` / `hostname`（组信、SMTP、Message-ID）

---

## 2026-07-24 — Windows 打包脚本中文/路径兼容

- `export-images.ps1` / `push-images.ps1`：UTF-8 BOM + `chcp 65001`；固定切到脚本目录；中文/`PROJECT_NAME` 自动映射为 Docker 可用的 ASCII 名；非 ASCII 路径告警；`docker info` 检测；退出改 `ReadLine`（避免 `pause` 异常）
- 生成的 `import-images.ps1` 同样带 BOM；指南注明 Windows 打包机路径建议英文

---

## 2026-07-24 — 编译优化阶段 A–C（不影响功能）

- **A**：README / `export-images.sh`·`export-images.ps1` / `push-images.sh`·`push-images.ps1` / `dev.ps1` 写清 BuildKit cache、预编译 ELF、JOBS、勿删 target
- **B**：`axum` 去掉未用的 `ws`/`macros`；`futures` 收窄为 `std`+`async-await`；支付 `rsa` features 与 `reqwest` gzip/stream/multipart 保留（Windows/Mac/Linux 共用 `Cargo.toml`）
- **C**：直接依赖再审计无新增可删项；TOS 仍传递 `reqwest 0.11`（上游约束，不强行统一）；不为 TOS/邮件加额外 feature 门控以免误伤默认能力

---

## 2026-07-24 — 收窄 Cargo features / 删除确认未用直接依赖

- `tokio`：去掉 `full`，保留 default + `fs`/`signal`（覆盖现有 spawn/time/sync/net/fs/signal 用法）
- 去掉直接依赖 `tower`（曾 `features=["full"]`）、`axum-extra`、`eventsource-stream`、`pin-project-lite`、`bytes`（源码无引用；tower/bytes 仍由 axum 等传递）
- `tower-http`：仅保留实际使用的 `cors`/`fs`；`tracing-subscriber` 去掉未用的 `json`
- 功能路径不变；本机 `./dev.sh` / 镜像构建均可直接验证

---

## 2026-07-23 — 计费路径小修补（行为不变）

- 预扣退钱包：赠送退回钳在 `cost` 内，避免脏数据把 balance 打成负向
- 异步结算令牌强制落账后 `invalidate` 内存 slot，防止限额满时内存与 DB 漂移

---

## 2026-07-23 — 修复超龄冻结退款 `integer out of range`

根因：`logs.latency_ms` 为 INT4，超龄冻结 `EXTRACT(EPOCH)*1000` 直接 CAST 触发 PG 22003，退款事务回滚、僵死任务被反复捞起。结算/退款共用钳位表达式，功能与退款金额不变。

---

## 2026-07-23 — 精简无用代码（不影响功能）

- 确认并保持删除：`forward` / `upstream_asset_relay` / `upstream_asset_client` 内单元测试模块
- 去掉 `QuotaSlot`/`QuotaLimits` 从未读取的字段；去掉 `asset_convert` 仅转发的薄包装函数
- 保留产品能力：通道测试、存储/邮件等「测试连接」接口

---

## 2026-07-23 — Relay：计费闭环 3–8 点加固

- TaskPoller：2 日窗口、`id ASC` 分批；超窗冻结兜底退款
- BillingPipeline：刷库失败短重试后放回缓冲
- 令牌：限额已满仍强制落账（同步 + 异步结算差额）；授信封顶后 `logs.cost`/额度/钱包对齐
- 追加扣款：`cap_additional_charge`；HA 复用 pending 时预扣 reopen 为处理中
- 日统计：仅 `is_completed=1`；audio/generic 预扣改到上游成功后

---

## 2026-07-23 — Relay：预扣与 pending 日志同事务落账

`pre_deduct` 在扣钱包的同一事务写入 `logs.cost` / `pre_deduct_gift`，避免崩溃后「钱已扣、日志 cost=0」无法退款。孤儿清理/启动恢复抽公共 `close_pending_and_refund`；结算落库失败时对 pending 预扣立即 CAS 退回。去掉无用 `PreDeductSplit`，赠送优先拆分抽到 `money::split_gift_first`，钱包退款 SQL 复用。正常成功/失败/冻结结算金额语义不变。

---

## 2026-07-23 — Relay：超管/管理员与普通用户同等计费

取消管理员余额校验豁免与预扣跳过：`check_access` / `pre_deduct_or_intercept` 不再因 `role=admin` 放行；钱包预扣、终态扣费、令牌/渠道额度与普通用户同一路径。并移除 `pre_deduct_or_intercept` 的冗余 `role` 参数，以及 `UserContext` 中已无用的 `role` 字段与查询列。

---

## 2026-07-23 — 火山视频转素材ID：优化插件展示名

插件展示名由「上游素材中转」改为「火山视频转素材ID」，描述对齐火山视频自动转素材 ID 场景；内部 `name=upstream_asset_relay` 与路由不变。直接改本地种子 SQL（无增量迁移）。

---

## 2026-07-23 — 上游素材中转：去掉冗余代码

插件内合并绑定筛选公共逻辑；缓存列表不再取未展示的 `file_name`；生成规则响应去掉前端未用的 `config_json`；uid_map 查询抽公共函数。行为不变。

---

## 2026-07-23 — 修复火山 MediaKit「使用日志」慢查询与空列表

根因：`enhance-logs` 按裸 `plugin_tag IN ('vve-sd',…)` 过滤，但落库实为 JSON 字符串 `"vve-sd"`（或空 tag + `model`/`model_id`），COUNT 全表慢扫、列表常返回空。

处理：改为 `model = ANY(mid+model_id)`（走 `idx_logs_model_created`）或 `action_type = '视频增强'`；列表用 `deferred_join_page_sql` + mid/model_id 双键 LATERAL 取模型名；新写入 `plugin_tag` 改为 `{"mid":…}` 便于与 cascade 合并。接口字段与详情弹窗行为不变。

---

## 2026-07-23 — 上游素材中转：转换日志改为只读

对齐素材资产插件 API 日志：去掉单条/批量删除与对应后端 `POST /convert-logs/delete`；筛选、分页、刷新与展开详情不变。缓存管理仍在「素材缓存」Tab。

---

## 2026-07-23 — 上游素材中转：新增「素材缓存」Tab

对齐素材资产管理：可分页查看/筛选/单删/批删/清空本插件 `plugin_assets`（source=`upstream_relay_convert`）；与「转换日志」分离。删除仅清本地缓存，不影响 API 日志与转换能力。

---

## 2026-07-23 — 上游素材中转：区分转换日志与素材缓存

转换日志只读 `plugin_api_logs`；`plugin_assets`（source=`upstream_relay_convert`）是复用缓存，命中缓存时也可能不再写新日志。缓存查看/删除见「素材缓存」Tab。

---

## 2026-07-23 — 上游素材中转：减少可复用场景的重复查库

生成规则：已 JOIN 出的 `rule_name` 判断规则是否仍在，写回后复用内存中的 binding，不再二次 `fetch_binding`。运行时转换：插件启用与绑定/渠道一次 JOIN。转换日志 `total=0` 时跳过列表与 uid_map 查询。

---

## 2026-07-23 — 上游素材中转：移除契约单测残留

删除 `upstream_asset_relay` / `upstream_asset_client` 中的 `#[cfg(test)]` 契约测试；转换日志 scope 改回静态常量，避免每次请求 `format!`；运行时绑定/转换/日志能力不变。

---

## 2026-07-23 — 上游素材中转：去掉多余 api.ts，对齐 relay 常量

删除仅含一行前缀的 `api.ts`；`LOG_SOURCE`/`PLUGIN_NAME`/`binding_ns` 收拢到 `upstream_asset_client`，转换日志与写入侧共用；`asset_convert` 抽出 `shorten_url_for_log`，上游调用改用模块别名。

---

## 2026-07-23 — 上游素材中转：低风险精简

生成规则写回绑定抽公共函数；转换日志绑定名用一次映射。

---

## 2026-07-23 — 上游素材中转：转换日志筛选与分页加固

按 `binding_id` 筛选共用日志范围常量；列表页码按 total 钳制；删除后自动回退有效页；去掉无用 `source` 字段返回；生成规则命中已有绑定时不再重复查询。

---

## 2026-07-23 — 上游素材中转：转换日志按上游关联筛选

转换日志支持按绑定（`binding_id` → `plugin_name=uar:{id}`）筛选；列表展示关联名称；后端查询仍限定本插件日志范围。

---

## 2026-07-23 — 上游素材中转：转换日志对齐展示并支持删除

转换日志时间用 `formatApiDateTime`、用户展示 UID+用户名（`uid_map`）；去掉「来源」列；支持单条/批量删除（仅限本插件 `upstream_relay_convert` / `uar:%` 日志）。

---

## 2026-07-23 — 素材转换 Range 元数据超时改为 10s

指纹 Range 请求超时调整为 10s（到点才失败，提前返回不等满）；失败类型日志仍区分超时等。

---

## 2026-07-23 — 素材转换 Range 元数据失败日志区分超时

`fetch_meta_fingerprint` 请求失败时日志标明超时 / 连接失败 / 请求发送失败 / 其它，便于确认是否撞 5s 上限；超时时间暂不调整。

---

## 2026-07-23 — 素材转换指纹改用 GET Range 取元数据

`fetch_meta_fingerprint` 由 HEAD 改为 `GET Range: bytes=0-0`：整文件长度取 `Content-Range` 总长（源站忽略 Range 回 200 时回退 `Content-Length`），ETag/Last-Modified 不变；指纹公式仍为 `SHA-256(URL路径|长度|ETag|Last-Modified)`，与历史缓存兼容。L1/L2 复用规则不变。

---

## 2026-07-23 — 素材转换缓存：恢复「指纹未命中不查 URL」

统一 `lookup_cached_converted_asset`（原 `asset_convert` / 上游中转共用）：L1 指纹命中才复用；指纹已算出但对不上则跳过 URL 并重新注册（防同 URL 内容变更误复用）；仅指纹算不出时走 L2 URL。写入缓存失败打 warn。

---

## 2026-07-23 — 素材转换 CreateAsset 补齐 Name

转发素材转换与上游素材中转 CreateAsset 均补传 `Name`（由 URL 文件名推导，缺省则短哈希名），避免代理/上游报「素材名称不能为空」；落库 `file_name` 同步复用同一推导。

---

## 2026-07-23 — 上游素材中转：修复绑定保存 500（SQL 占位符）

根因：`fetch_binding` 追加 `WHERE id=?` 未走 `format_query`。统一列表/单条查询占位符转换；绑定字段沿用 `channel_config_id`（本地库已对齐）。Relay 仍 JOIN `channel_configs` 取 base_url/api_key。

---

## 2026-07-23 — 上游素材中转：关联列表对齐渠道预设展示与搜索

绑定列表/下拉改用「上游渠道配置预设」信息（名称、YID、服务商、base_url），支持按名称/YID/服务商/地址搜索快速添加；接口补充返回 `config_yid` / `config_provider_type`。仍绑定 `channel_configs.id`，不改运行时转换与旧 `asset_convert`。

---

## 2026-07-23 — 上游素材中转：清理验证单测残留

删除 `upstream_asset_relay` 落地时临时加入的单元测试（客户端 URL 拼装、规则 JSON、转发字段解析），保留管理端绑定/生成规则/运行时转换与日志能力；与现网 `asset_convert` 隔离不变。

---

## 2026-07-23 — 本地后端增量编译/链接加速（兼容 Windows）

- `[profile.dev]`：`debug = line-tables-only` + incremental。
- Windows：`dev.ps1` 默认设 `rust-lld`；`TOKENSBYTE_FAST_LINK=0` 回退 `link.exe`（不新增 `.cargo/config.toml`）。
- Linux：`dev.sh` 有 mold 则 `mold -run`，否则可选 `clang+lld`；macOS 不启用（`mold -run` 不支持）。
- 不改业务逻辑；无额外单测残留。

---

## 2026-07-23 — 级联转发规则：每分辨率可配增强版本与底座

- `config_json` 新增 `res_enhance`（fast|standard|pro|ai，默认 standard）、`res_scene`（标准版场景 common|ugc|short_series|aigc|old_film，默认 common）、`res_base`（阶段一座底，默认一级：720p→480p、1080p→720p、2k/4k→1080p）。
- 管理端级联配置按分辨率设置倍率/增强/场景（仅标准版显示）/底座；阶段二标准增强透传并校验 `scene`。
- 旧规则无新字段时增强默认标准、场景 common、底座一级，无需迁移；不保留级联相关验证单测残留。
- 级联 `version` / 目标 `resolution` 只写入 `plugin_tag.cascade`，不改用户入参；预扣费时用已有 `cascade_json_str` 写入 `billing_features`，终态直接复用快照。

---

## 2026-07-22 — 任务列表预览：列表返回产物地址，不受详情权限限制

根因：列表不带响应体，预览走详情接口；关闭「查看日志详情」时取不到媒体链。  
处理：`/task_logs` **一次查询**仅对图片/视频/视频增强读 `response_content`，内存提取 `preview_urls`（级联取 stage2，复用 `find_urls`，只回传 http(s)；日志内 base64 本已脱敏占位）后丢弃大字段；前端优先用 `preview_urls`，无详情权限时不再打详情兜底。详情权限仍约束完整展开，预览不受影响。

---

## 2026-07-22 — 新增系统增强插件「上游素材中转」

新增系统增强插件 `upstream_asset_relay`：可多条关联上游渠道配置与素材基础路径（路径可空），一键生成视频转发规则；模型选用后对请求内图片/音视频 URL 经渠道 Bearer 调用 CreateAsset/GetAsset 转为 `asset://`，带缓存与 `plugin_api_logs` 追溯。与现网 `asset_convert`（素材插件凭证）正交隔离，默认关闭，不影响原有行为。

---

## 2026-07-22 — 启动/定时清理：status_code=0 慢查询改走部分索引

`recover_interrupted_logs` / `cleanup_orphan_pending_logs` 原 SQL 对 400 万行 `logs` 全表扫（`status_code=0` + `NOT ILIKE '%冻结%'`，约 2s）。  
预记录默认 `is_completed=0`，改为先命中 `idx_logs_is_completed_pending` 再过滤；语义不变（异步冻结多为 `status_code=200`）。实测约亚毫秒级。

---

## 2026-07-22 — 修复 DbGate 卡 Loading structure（锁 + 损坏索引）

根因：多实例 StartupBackfill 长事务占 `logs` 锁，迁移非并发 `DROP INDEX` 与 DbGate 结构查询一起等待；另有损坏索引 `idx_logs_action_created_stats_new`（pg_attribute 缺口）干扰元数据。  
处理：运维侧已清损坏目录项并补齐 `idx_logs_created_at_agg`；`logs_indexes_reconcile_v1` 的 prune 增加 `lock_timeout=3s`，拿不到锁则跳过，不再堵库。

---

## 2026-07-22 — 去掉 migrations 验证单测

删除 `idempotent_index_ddl_tests`（`#[cfg(test)]` 不影响线上，但仓库不留验证残留）；幂等判断内联为单一私有函数，迁移终态与行为不变。

---

## 2026-07-22 — logs 索引迁移收口为 logs_indexes_reconcile_v1

- 旧 ID `logs_slow_query_indexes_v1` / `logs_created_at_agg_prune_v1` 改为 no-op（保留 history 兼容）。
- 新迁移 `logs_indexes_reconcile_v1` 为唯一终态：清 INVALID → 建 `idx_logs_created_at_agg` / `idx_logs_vision_created_at_new` → 尽力删冗余/损坏旧索引 → `ANALYZE`。
- `once_migration!` 仍对 CREATE 名冲突（23505）与 DROP 目录缺口（XX000）幂等跳过。重启后端执行一次即可。

---

## 2026-07-21 — 今日改动复查收口

- 腾讯视频：用户显式 `LastFrameUrl` 原样透传（可与 `FileInfos` 并存）
- 去掉 `PollTask` 内嵌套 `if is_tencent`
- 级联阶段二不再默认写 `bitrate_level: high`（恢复历史请求体，避免静默改变上游画质/计费）

---

## 2026-07-21 — 腾讯云 FileInfos 用户原样透传

用户已传 `FileInfos` 时图/视频 body 直接 `clone`，不做规范化改写；仅从 `images` 等兼容字段构建时仍走 `tc_file`。已删 `tc_norm_files`。

---

## 2026-07-21 — logs 日聚合索引补齐与冗余索引精简

- 新迁移 `logs_created_at_agg_prune_v1`（**未并入**已上远程的 `logs_slow_query_indexes_v1`，避免已执行环境跳过删冗余）
- 补齐 `idx_logs_created_at_agg`；删 `idx_logs_action_created_stats_new` 及旧名 `idx_logs_created_at*`（若有）
- 保留：pkey / log_id / task_id / user_id / action_created / vision / is_completed / status0；重启后端执行

---
## 2026-07-21 — 消除 LocalDayBounds.local_day dead_code 警告

`local_day` 仅赋值从未读取（调用方用 `start/end_utc` 或 RFC3339）；从结构体移除，边界计算不变。

---

## 2026-07-21 — 腾讯云 FileInfos 支持 Base64 输入

兼容字段（`images` 等）构建 `FileInfos`：base64 → `Type=Base64`+纯串，否则 `Url`（`tc_file`）。用户自带 `FileInfos` 见上方「原样透传」。

---

## 2026-07-21 — 清理无用单测与临时脚本

删除 `live_metrics` / `dashboard` 内 `#[cfg(test)]` 模块，以及根目录孤儿脚本 `test_plugins.js` / `test_plugins.sh`。不改业务路径；保留管理端「通道测试」等产品功能。

---

## 2026-07-21 — 级联阶段二防重复：DashMap 互斥

同进程 `cascade_s2_inflight` + RAII Guard（Drop 必 remove）。输家零查询 `InProgress`（对外仍走标准「进行中」）；无额外读库。裁剪固定角点 `(2,6)-(862,490)` / `(6,2)-(490,862)`。单实例部署，不做跨进程 CAS。

---

## 2026-07-21 — 手动轮询补齐腾讯云原始响应日志

根因：仅后台 `[TaskPoller]` 打印腾讯原始 body；用户手动 `[Task Poll]` 只打 `resp_len`。抽出 `log_tencent_poll_raw`，手动/自动/`PollTask` 共用；`is_tencent` 统一 `starts_with("tencent_vod")`。仅日志，结算与返回不变。

---

## 2026-07-21 — 腾讯视频终态分辨率覆盖计费特征

结算仅读终态 `Output.FileInfos[0].MetaData` 的 `Duration` / `Width` / `Height`（短边→480p/720p/1080p/2k/4k），覆盖请求侧任意分辨率；不读空/不可靠的 `Resolution`。在 `merge` 之后写入，避免被冲掉。

---

## 2026-07-21 — 级联阶段一 480p 非标居中裁剪

级联模式下阶段一为 480p 且 ratio 为 16:9/9:16 时，超分前走 MediaKit `crop-video` 居中裁成标准 480p（864×496→860×484 / 496×864→484×860）。逻辑在 `cascade.rs`，`task.rs` 阶段二提交复用；不命中不裁，失败回退原底座。

---

## 2026-07-21 — 启动慢 SQL：日统计回填 + 日志深翻页

### 现象
- 启动 `StartupBackfill`：`INSERT…SELECT` 聚合 `logs → usage_daily_stats` 超 1s（周批次可扫数十万行）。
- 日志列表：视觉类筛选 + 大 `OFFSET` 时，对 OFFSET 前全部行做 5 表 JOIN 与 `regexp_match(billing_detail)`。

### 改动（行为不变）
- `usage_stats`：按日 upsert；SQL 循环外编译复用；`FILTER` + `GROUP BY 1..6`；失败中止；仅日间 sleep。
- `list_logs` / `list_task_logs`：共用 `deferred_join_page_sql`；视觉 `= ANY(...)`；排序仅 `created_at DESC`。
- 迁移 `logs_slow_query_indexes_v1`：`idx_logs_created_at_agg`、`idx_logs_vision_created_at_new`。
- 验证用临时单测已删除，不留仓库残留。

---

## 2026-07-21 — 打包脚本精简原则

- **不抽** `docker-build-env` 公共文件（曾引入引用/漏改风险）；sh 与 ps1 各自独立、规则对齐即可。
- **不加** Docker 打包常驻单测（需本机 Docker/长编译，易碎；验证用 `bash -n` + 实跑导出）。
- `push-images.sh` 与 `export-images.sh`：Apple Silicon 默认 arm64，避免推送路径默认踩 QEMU。

---

## 2026-07-21 — 本地打包提速（不影响线上）

### 原则
只改构建过程；运行时镜像仍是 release Linux ELF + nginx，部署/更新方式不变。

### 改动
- 构建顺序：`frontend` → `backend`（先轻后重，给 Rust 腾内存）。
- 编译阶段启用 `lld`（仅链接加速）。
- `EXPORT_FAST=1` → Mac 上 JOBS=2；`SKIP_BUILD=1` → 只导出已有镜像。
- 二次构建依赖已有 BuildKit cargo/npm cache（勿随意 `builder prune`）。

### 使用
```bash
EXPORT_FAST=1 ./export-images.sh   # Mac 提速（内存够时）
SKIP_BUILD=1 ./export-images.sh    # 已有镜像只打 tar
# 正式发版：CI 构建推送 → 服务器 pull（本机最省时间）
```

---

## 2026-07-21 — Mac 导出修复：`no such service: 1` + OOM 加固

### 根因
- `docker compose build --parallel 1` 在 Compose v5 无效：`--parallel` 不是 build 选项，`1` 被当成服务名 → `no such service: 1`。
- 另：Mac Desktop ~8GB 时 `JOBS=2` 易 OOM（与架构无关）。

### 改动
- 导出/推送改为显式串行：`docker compose build backend` → `frontend`（Mac/Windows 一致）。
- Mac 默认 `CARGO_BUILD_JOBS=1`；Windows 默认 2；交叉架构强制 1 + 关 Cargo cache。
- Dockerfile / compose 安全默认 jobs=1。

### 使用
```bash
./export-images.sh                    # Mac：选 linux/arm64；JOBS=1；串行构建
CARGO_BUILD_JOBS=2 ./export-images.sh # Desktop 内存 ≥12GB 时可试
.\export-images.ps1                   # Windows 默认 JOBS=2；OOM 时 $env:CARGO_BUILD_JOBS=1
```

---

## 2026-07-20 — Docker 镜像构建降内存 / 提速（Mac·Windows·Linux）

### 根因
- `codegen-units=1` 推高 LLVM 峰值，Docker Desktop 易 OOM；无 BuildKit cache 导致反复全量编译。
- `Cargo.lock` 曾被 dockerignore 误排除；导出脚本 Darwin 分支缺 `fi`。
- 复查补修：跨架构 cache 未按 `TARGETARCH` 隔离（amd64/arm64 可能串缓存）；`set -e` 下无效的 `$?` 检查。

### 改动
- `backend/Dockerfile`：按架构隔离 BuildKit cache、`CARGO_BUILD_JOBS`；运行时仍 debian bullseye。
- `frontend/Dockerfile`：`npm ci` + npm cache；nginx 不变。
- `Cargo.toml` release：`opt-level=3`、`codegen-units=16`。
- 导出/推送脚本内联 BuildKit、平台与 jobs；交叉架构强制 jobs=1。

### 使用
```bash
./export-images.sh
DOCKER_DEFAULT_PLATFORM=linux/arm64 ./export-images.sh
.\export-images.ps1
```
勿用 Mac 本机二进制替代镜像内程序；导入与 `docker compose up -d` 不变。

---

## 2026-07-20 — 系统概览性能优化

- 实时吞吐轮询 2s → 5s；`/metrics/live` 鉴权仅校验 JWT，跳过 `is_active` 查库
- 去掉日期快捷「全部」；RangePicker 不可清空（避免无界全表聚合）
- `dashboard_cache` 后台每 5 分钟清理超过 30 分钟的缓存条目

## 2026-07-20 — 系统概览仪表盘布局精简

- 请求数与总令牌合并为一张卡片；QPS / RPM / TPM / Task 合并为一张「实时吞吐」卡片
- 消耗 Token、预估成本卡片保持不变；去掉原先独立的 4 格实时吞吐行

## 2026-07-20 — 系统概览默认展示今日数据

- 控制台「系统概览」打开时，日期快捷选项与区间默认从「本月」改为「今日」。

## 2026-07-20 — Dashboard 实时吞吐观测（QPS/RPM/TPM/Task）

- **能力**：系统概览页顶部新增实时吞吐条（QPS / RPM / TPM / Task）；Admin 看全局，普通用户看本人所有 API Key 汇总
- **后端**：`middleware/live_metrics.rs`（static 原子全局 + DashMap 按 token 分槽 + 双 RAII Guard）；埋点在 `api_key_middleware`、`record_and_bill_inner` 与异步 `execute_settlement_tx`；`GET /api/v1/metrics/live`；冷用户 1h TTL / 5min 清理
- **约束**：P0 仅观测不限流；热路径无 Mutex；不写库
- **前端**：`Dashboard.tsx` 独立实时条，2s 轮询，页面隐藏时暂停

---

## 2026-07-20 — dev 启动支持后台 / 前台日志双模式

### 改动
- `dev.sh` / `dev.ps1`：默认 **后台**；`fg` / `DEV_ATTACH=1` 为 **前台日志**（Ctrl+C 仅停本实例）。
- 精简：去掉 `DEV_FAST` 预起二进制分支，统一 `cargo watch -x run`；复用 `port_in_use` / `follow_log`。
- 用法：`./dev.sh [1|2] [bg|fg]`、`.\dev.ps1 [1|2] [bg|fg]`。

### 使用
```bash
./dev.sh                 # 后台（默认）
./dev.sh fg              # 前台日志
.\dev.ps1                # 后台（默认）
.\dev.ps1 1 fg           # 前台日志
```

---

## 2026-07-20 — 加快本地开发启动（兼容多实例 / Windows）

### 改动
- `backend/Cargo.toml`：`[profile.dev]` 使用 `debug = "line-tables-only"`，缩短链接耗时（不影响 release / 业务逻辑）。
- `dev.sh` / `dev.ps1`：统一 `cargo watch -x run`；去掉易出端口冲突的「预起二进制 + postpone」快启分支，增量交给 Cargo。
- 多实例：仍按路径哈希隔离 state、端口避让、只回收本仓库进程；各 checkout 默认各自 `target`（Windows 非 ASCII 路径仍重定向到 `%LOCALAPPDATA%`）。

---

## 2026-07-20 — 对齐半开日期边界与前端绝对时刻传参

### 改动
- 后端：`parse_instant_bound` / `parse_timestamptz_bind` / `push_timestamptz_bound`，纯日期与无偏移时刻按 timedisplay 半开；终点 `< ?::timestamptz`。
- 修复：无偏移 `YYYY-MM-DD HH:mm:ss` 不再被误当成纯日期截断到 00:00。
- 前端：`dateRangeParams.ts`；管理端 usage-stats 同步也改传 ISO。
- 精简：去掉未用 `AbsoluteRange` / coarse 二元组 / `sql_timezone_convert` / 废弃上海边界函数；删除 `date_helper` / `time_system` 内单元测试样例；前端日期传参统一走 `dateRangeParams`（含 AdvancedMarketing / Settings）。
- 修复：日志详情 `get_log_detail` 对非管理员补齐级联脱敏（列表大字段本为 NULL，原先 sanitize 无效）；`dateRangeParams` 拦截 Invalid Date；去掉未接线的 `time_system/package` 与死代码。

---

## 2026-07-20 — 恢复 dev.sh 多实例兼容

### 改动
- `dev.sh`：保留「后台拉起、就绪退出」体验；按目录名设 `PROJECT_NAME`；复用已运行 Postgres；前后端端口占用时顺延；仅清理本仓库残留进程，不误杀其它目录实例。
- 等待就绪改为最长 600s，并每 15s 打印编译进度，避免首次重编译被误判为卡死。

### 使用
```bash
./dev.sh          # 本地后台（默认）
./dev.sh 2        # Docker 全容器
# 可选：PROJECT_NAME / BACKEND_PORT / FRONTEND_PORT / POSTGRES_PORT / DEV_WAIT_MAX
```

---

## 2026-07-20 — 创作中心时间存储与展示全量核对

### 结论
创作中心库表时间列已在 `timestamptz_unify_v1` 覆盖（`playground_projects` / `playground_assets` / `user_model_configs`）；API 读写用 `DbTs` + `NOW()` / `?::timestamptz`。本次补齐前端展示与画布 JSON 时间一致性，以及清理任务的时间解析。

### 后端
- `list_projects` / `get_project` / assets / `user_model_configs`：`created_at`/`updated_at` 用 `DbTs` 解码（避免 TIMESTAMPTZ→String 崩溃）
- `cleanup_stale_playground_nodes`：画布 `created_at` 用 `parse_flexible_ts`；日志匹配绑定 `DbTs` + `?::timestamptz`

### 前端
- 统一经 `parseApiTimeAsUtc` / `formatApiDateTime`（timedisplay）：项目列表、悬浮头、创作日志、资源管理、节点详情、Token 弹窗
- 画布 `taskData.completed_at` 由毫秒时间戳改为 ISO 字符串，与 `created_at` 一致
- 资产回填/超时判定按 UTC 解析，不再依赖浏览器本地 `new Date(无偏移字符串)`

### 部署
前后端同步发布；后端需重启。无新迁移。

---

## 2026-07-20 — 复查：补修 playground TIMESTAMPTZ 解码为 String

### 问题
日志：`decoding column 7: String not compatible with TIMESTAMPTZ`。
创作中心项目列表 SQL 第 7 列是 `created_at`，却用元组 `String` 解码。

### 改动
`playground.rs`：`list_projects` / `get_project` / assets / `user_model_configs` 改为 `DbTs`。

### 部署
重启后端生效。

---

## 2026-07-20 — README 精简与约定补齐

### 改动
- 重写根目录 `README.md`：去掉重复营销与过时章节，保留部署 / 开发 / 运维要点
- 修正管理员默认密码表述不一致；补齐金额 6 位小数、TIMESTAMPTZ、日志归档约定
- 变更历史仍以本文件为准，README 仅作入口链接

### 同步小修
- 渠道配置额度展示、公告低余额阈值输入改为 6 位小数
- `money::format_money` 供通知等格式化复用，消除未使用常量告警

---

## 2026-07-20 — 金额精度统一为小数点后 6 位

### 约定
站点内部账本（日志 cost、扣费结算、余额/赠送金/信控、充值调账、额度用量）一律保留 **6 位小数**（四舍五入）。
支付通道对外法币金额（微信/支付宝等）仍按通道要求保留 2 位，不在此范围。

### 改动
- 后端新增 `money::round_money`，接入余额 API、管理员充值、计费结算、预扣拆分、额度微单位
- 前端日志/财务/钱包/用户/令牌/渠道/仪表盘等金额展示统一 `toFixed(6)` / `precision={6}`
- 前端金额展示统一 `toFixed(6)` / `precision={6}`（与后端 `money::round_money` 对齐）

### 部署
前后端同步发布；无新库迁移。

---

## 2026-07-20 — 安全加固：代登鉴权 / OAuth State / 验证码防爆破

### 问题（安全检测 P0）
1. 代登接口 handler 未显式校验管理员 Claims（虽路由层有 middleware，缺少防御纵深）
2. OAuth state 存在过松兼容路径：任意 `wechat_XXXXX` 可通过校验，绕过 HMAC
3. 邮箱/短信验证码无尝试次数限制，6 位数字可暴力破解

### 改动
- `impersonate_user`：强制 `role == admin`，并记录审计日志
- 删除 OAuth state 前缀兼容分支；仅接受服务端 HMAC 签发；登录页/注册页改为请求 `/auth/oauth/state`
- 绑定/换绑微信与谷歌 state 同样改为 HMAC 签发（`/user/bind/oauth-state`）
- `verification_codes` 新增 `attempts` 列；错误超 3 次作废；有效期改为 5 分钟

### 部署
重启后端以执行迁移 `verification_codes_attempts_v1`；前端需同步发布。

---

## 2026-07-20 — 修复 TIMESTAMPTZ 与 TEXT 比较导致 Internal database error

### 问题
列改为 `TIMESTAMPTZ` 后，部分接口仍用字符串参数做 `created_at >= ?`，PostgreSQL 报错：
`operator does not exist: timestamp with time zone >= text`。
前端表现为：日志/任务列表有时能出数据，同时弹出 **Internal database error**（并行 COUNT/列表或其它带时间筛选的接口失败）。

### 改动
所有对 `TIMESTAMPTZ` 列的范围比较统一为 `?::timestamptz`（含 logs 已有路径、dashboard / finance / auth / user wallet / team_marketing / happyhorse / `date_helper::sql_cond`）。

### 部署
重启后端使新二进制生效即可（无新迁移）。

---

## 2026-07-19 — 时间体系统一：线上自检 / 财务用户展示 / logs 归档

### ① 线上自检 SQL（TIMESTAMPTZ）

在业务库执行，确认迁移 `timestamptz_unify_v1` 已落地：

```sql
-- 1) 迁移是否执行
SELECT id, executed_at FROM sys_migration_history
WHERE id IN ('timestamptz_unify_v1', 'logs_archive_v1')
ORDER BY id;

-- 2) 关键表时间列类型（期望 timestamp with time zone）
SELECT table_name, column_name, data_type
FROM information_schema.columns
WHERE table_schema = 'public'
  AND (
    (table_name, column_name) IN (
      ('logs', 'created_at'),
      ('users', 'created_at'),
      ('users', 'updated_at'),
      ('orders', 'created_at'),
      ('orders', 'paid_at'),
      ('recharge_records', 'created_at'),
      ('commissions', 'created_at'),
      ('verification_codes', 'expires_at')
    )
  )
ORDER BY table_name, column_name;

-- 3) 仍为 text/varchar 的业务时间列（理想应为空；周期键 last_reset_* 除外）
SELECT table_name, column_name, data_type
FROM information_schema.columns
WHERE table_schema = 'public'
  AND column_name ~ '(created_at|updated_at|paid_at|expires_at|last_used_at|expire_at)$'
  AND data_type IN ('text', 'character varying')
  AND column_name NOT LIKE 'last_reset%'
ORDER BY table_name, column_name;

-- 4) logs 热表体量与索引
SELECT relname, n_live_tup, n_dead_tup
FROM pg_stat_user_tables WHERE relname IN ('logs', 'logs_archive');

SELECT indexname FROM pg_indexes
WHERE tablename = 'logs' AND indexname LIKE '%created%';
```

### ② 财务 / 用户页展示

- `formatApiDateTime` 改为按 **timedisplay**（用户时区 > 站点默认）直接格式化，不再依赖浏览器本地时区。
- 已切换：`GiftRecords` / `RechargeRecords` / `Users` / `AdminGroups` / `UserLevels`（订单详情此前已用）。

### ③ logs 分区 / 归档方案

**已落地（Phase 1 — 冷表归档）**

| 项 | 说明 |
|---|---|
| 表 | `logs_archive`（`LIKE logs` + `archived_at`） |
| 开关 | `storage_settings.log_row_retention_days`（默认 **0**=不归档） |
| 时机 | 每天凌晨详情清理后，分批每批 5000 行：先 INSERT 冷表再 DELETE 热表 |
| 缓冲 | 实际阈值 = 配置天数 **+2**，降低统计未落档风险 |
| 建议 | 大数据量站点：先校准 `usage_daily_stats`，再设 `90`；且 ≥ `log_retention_days` |

**Phase 2 — 原生月分区（运维窗口，按需）**

热表仍过大时，在维护窗口把 `logs` 改为 `PARTITION BY RANGE (created_at)`。要点：

1. 新建分区父表 + 按月子表（含未来 2～3 个月）。
2. `INSERT INTO logs_partitioned SELECT * FROM logs`（或按月批次）。
3. 交换表名 / 重建索引 / 切应用；旧表改名备份后再 DROP。
4. 冷表 `logs_archive` 可同样按月分区，或按年 DETACH 后迁对象存储。

> 未自动执行 Phase 2：线上改分区需短时锁表与校验，请单独排期。

### 部署

停旧后端 → 启新二进制跑迁移（含 `logs_archive_v1`）→ 管理端按需设置「日志行归档天数」→ 再开流量。

---

## 2026-07-19 — 重置密码/注册：获取验证码防刷

### 问题
重置密码页「获取验证码」可连点，`sendingCode` 为异步 state 拦不住并发请求，失败时也不开倒计时，导致错误弹窗刷屏。

### 改动
- `ForgotPassword` / `Register`：`useRef` 同步锁 + `cooldownUntilRef` 时间戳冷却，堵住 state 提交竞态。
- 成功冷却 60s；失败短冷却 3s；切换找回方式不再清零倒计时。
- 请求中禁用按钮并显示 loading。
- 后端已有 `check_code_send_cooldown`（60s），前端防刷主要解决弹窗刷屏。

### 涉及文件
- `frontend/src/pages/Login/ForgotPassword.tsx`
- `frontend/src/pages/Login/Register.tsx`

## 2026-07-19 — 站点时间体系统一（落库 / 查询 / 展示）

### 改动
- `once_migration!`：任一句失败不写 history，支持重启重试。
- TIMESTAMPTZ 写入统一为 `DbTs::now()` / `CURRENT_TIMESTAMP`（去掉 `::text`、朴素字符串绑定时戳列）。
- 注册邀请日限额 / IP 日限额：`created_at LIKE` 改为站点 timedisplay 自然日 `[start, end)` 范围查询。
- 验证码校验：用 `expires_at > NOW()`，不再做字符串字典序比较。
- 前端 `timedisplay.ts` 导出 `formatApiDateTime` / `parseApiTimeAsUtc`；日志、订单、快乐小马等展示统一走该函数。

### 部署
停旧后端 → 启新二进制跑迁移 → 再开流量；勿与旧二进制混跑。

## 2026-07-19 — 全库时间列 TEXT → TIMESTAMPTZ

### 问题
绝大多数 `created_at`/`updated_at` 等以 TEXT 存储，logs 等热路径频繁 `::timestamptz` 转换，btree 索引难以有效服务时间范围查询，日志性能已顶不住。

### 改动
- 新增 `DbTs`（`TIMESTAMPTZ` ↔ API RFC3339 字符串），FromRow 模型时间字段统一改用该类型。
- 一次性迁移 `timestamptz_unify_v1`：业务时间列改为 `TIMESTAMPTZ`（周期键 `last_reset_*` 仍为 TEXT）。
- logs/dashboard/清理/归档等查询去掉列上 cast，改为对参数 `?::timestamptz`，便于走索引。
- 运行时写入将 `now()::text` 改为 `NOW()`。

### 部署注意
`logs` 大表 `ALTER TYPE` 会重写表并短时锁表，请安排维护窗口后重启后端以执行迁移。

### 涉及文件
- `backend/src/time_system/db_ts.rs`（新增）
- `backend/src/db/migrations.rs`
- `backend/src/models/*`、`backend/src/api/logs.rs`、`dashboard.rs`、`date_helper.rs` 等

## 2026-07-19 — 系统概览：日期口径对齐与范围标签

### 问题
管理端/用户端共用仪表盘，但「总*」大数字随筛选变化，今日/昨日副行与模型「近三天」却固定日历日，默认又落在「今天」，易被误判为数据错误。

### 改动
- 默认筛选改为「本月」，并恢复「全部」；有筛选时主指标文案跟随上方快捷标签（如「本月请求数」），今日/昨日副行仅在「今天/昨日」快捷项下显示。
- 标题旁标明数据范围：管理员「全站」/ 用户「仅本人」；管理端最近活动增加用户列。
- 模型明细近几日改为锚定筛选区间末日（≤ 今天）向内最多 3 天；「全部」仍为日历近 3 天。
- 最近活动查询关联 users，填充昵称/UID。

### 涉及文件
- `frontend/src/pages/Dashboard/Dashboard.tsx`
- `frontend/src/locales/zh.json` / `en.json`
- `backend/src/api/dashboard.rs`
- `backend/src/api/date_helper.rs`

## 2026-07-18 — 日志记录 / 任务列表：防连点与大数据量性能优化

### 问题
查询 / 重置 / 刷新在数据未返回时被疯狂点击，会叠加大量请求，浏览器与后端易被打崩；列表接口还把请求/响应大字段整包返回，数据量一大就更慢。

### 改动
- **前端**：新增 `QueryGuard`（新请求取消旧请求 + AbortController）；查询/重置/刷新按钮 loading 时禁用；取消中的请求不报错。
- **后端列表瘦身**：`/logs`、`/task_logs` 列表不再返回 `request_content` / `response_content` / `post_response` / `upstream_req_content`。
- **按需详情**：新增 `GET /logs/{id}/detail`；表格展开行与任务预览时再拉取大字段。
- **并行查询**：列表 COUNT / 数据 /（日志）汇总改为 `tokio::join!` 并行执行。

### 修复（同日）
- 去掉 1.2s 时间节流与“操作过于频繁”提示（StrictMode 重挂载会误伤首屏）。
- 全局 axios 拦截器忽略主动取消的请求，避免误报 `Network error`。

### 涉及文件
- `frontend/src/utils/queryGuard.ts`
- `frontend/src/utils/request.ts`
- `frontend/src/pages/Logs/Logs.tsx`
- `frontend/src/pages/Logs/TaskLogs.tsx`
- `backend/src/api/logs.rs`
- `backend/src/api/task_logs.rs`
- `backend/src/api/mod.rs`
- `backend/src/models/log.rs`
