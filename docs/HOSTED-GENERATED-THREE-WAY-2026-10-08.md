# 在托管 runner 上消费原产物的生成三方对照

## 输入、隔离和证据边界

本机WSL仍有旧运行器的内核等待，不能反复启动或用root替代UID10001验收。新专项直接消费已经成功验收的AMD64原生镜像，不重建、换JAR、接入宿主Chrome、连接生产或上传用户副本。运行器明确限定为GitHub托管Linux AMD64，检查系统[官方运行器环境字段](https://docs.github.com/en/actions/reference/workflows-and-actions/variables)。

原版JAR从公开3.2.14镜像中只读提取、容器不启动；必须完整SHA匹配未动的本机原件 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`，不能用标签相似认定同版。提取副本与候选JAR均只读挂载。候选源运行必须属于本仓库、六实际Native作业全部成功，source与tested snapshot文件相同；下载前核验，下载后沿用正式发布导入器复核JAR、归档、镜像及UID身份。

仅运行现有生成样本：原版JAR＋固定历史WebView、恢复JAR＋同历史WebView、恢复JAR＋内置Camoufox。历史renderer版本只作为固定参考，不证明曾是用户生产版本。两个容器共享独立无外网回环命名空间，没有发布端口或生产挂载；历史renderer实际停止后才能运行Camoufox。aggregate slice总限2 CPU／2 GiB／零swap／512 PID，每容器256 PID；不重置累计峰值或放宽旧门禁。最后额外查询精确所有权标签与cgroup后代，剩余必须为0。

下载／导入在托管宿主上进行，不是Reader RSS或运行预算。最多上传小型生成JSON，不上传原版或候选JAR、镜像归档、完整日志、账号目录、真实Cookie或正文。SSH、registry凭据和生产端口均不使用；只授予Actions读取权限。

## 两种模式必须分开

- `metadata`：原五个搜索／脚本／POST／Cookie旅程再加生成延迟DOM的真实getBookInfo，比较完整ReturnData／静态JSON及已有明确默认时钟窗口。不读取真实起点正文，不把生成HTML当成真实认证。
- `utf8-characterization`：保留六个生成UTF-8观测，使用已有诊断模式。历史截断与Camoufox正确值即使符合已知差异也必须非零退出、overallAccepted=false；不得用continue-on-error、预期失败绿色包装或诊断模式宣布严格兼容。

测试结果从本次实际运行读取；入口和生成元数据单测通过不等于三方已执行。更不能把一次生成metadata通过扩大为全部原始JAR／真实书源／登录／正文差分通过。

## 本地检查与已验收的来源

13个新的生成守卫实际通过；本机Python395项／1平台跳过／30.364秒，发布安全Node82／零跳过通过，工作流YAML已解析。实际Native37783548430/source2f8b37aa/tested ba1817b4的API前置检查已通过，但专项尚未托管执行。该来源自己的四普通工作流与八设置图已独立验收，见[展示修复](HTTP-TTS-DISPLAY-2026-10-08.md#展示增量自己的托管实跑已完成)。不借它们证明新专项成功。

GitHub手动入口为已有 `browser-image.yml`，设置 `three_way_native_run=37783548430`、`three_way_source=2f8b37aa1c786d87e84d9ac6b745105aa35972a4`、`three_way_revision=ba1817b4256bfde0b0019ee8861d99af18c037a0`，另选一种模式；其他公网／probe／soak模式必须为空／false。具体代码checkout与被测镜像revision分开记录。先执行metadata，完成后再按新事实决定下一专项，不并发重复跑旧失败网站条件。

## 已知风险与回滚

公开归档必须仍可下载且SHA相同，否则停止，不上传私有原件绕过。Docker必须已经使用systemd cgroups；磁盘不足时拒绝，不清理未拥有镜像或修改daemon。GNU timeout先TERM使原harness按所有权清理，超时／清理观察失败不算成功。原JAR启动／参考引擎兼容、最新样本和托管时序均可能暴露新失败，应保留而不是删用例或放宽预算。

回滚仅不再选择新专项或撤回新入口／驱动／生成守卫；不撤销已接受的前端修复，不改部署或原JAR，不删除用户报告。真实认证、本机精确运行、最终三方全部范围、版本提升和正式发布仍未完成。
