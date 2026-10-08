# 在托管 runner 上消费原产物的生成三方对照

## 输入、隔离和证据边界

本机WSL仍有旧运行器的内核等待，不能反复启动或用root替代UID10001验收。新专项直接消费已经成功验收的AMD64原生镜像，不重建、换JAR、接入宿主Chrome、连接生产或上传用户副本。运行器明确限定为GitHub托管Linux AMD64，检查系统[官方运行器环境字段](https://docs.github.com/en/actions/reference/workflows-and-actions/variables)。

原版身份门禁尝试从公开3.2.14镜像只读提取、容器不启动；必须完整SHA匹配未动的本机原件 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`，不能用标签相似认定同版。**本次实际失败：公开归档不是同一JAR，不能用它执行原件三方；这个来源现已停止重复尝试。**只有获得准确原件且有相应授权后，才可能继续下述业务路径。候选源运行必须属于本仓库、六实际Native作业全部成功，source与tested snapshot文件相同；下载前核验，下载后沿用正式发布导入器复核JAR、归档、镜像及UID身份。

仅运行现有生成样本：原版JAR＋固定历史WebView、恢复JAR＋同历史WebView、恢复JAR＋内置Camoufox。历史renderer版本只作为固定参考，不证明曾是用户生产版本。两个容器共享独立无外网回环命名空间，没有发布端口或生产挂载；历史renderer实际停止后才能运行Camoufox。aggregate slice总限2 CPU／2 GiB／零swap／512 PID，每容器256 PID；不重置累计峰值或放宽旧门禁。最后额外查询精确所有权标签与cgroup后代，剩余必须为0。

下载／导入在托管宿主上进行，不是Reader RSS或运行预算。最多上传小型生成JSON，不上传原版或候选JAR、镜像归档、完整日志、账号目录、真实Cookie或正文。SSH、registry凭据和生产端口均不使用；只授予Actions读取权限。

## 两种模式必须分开

- `metadata`：原五个搜索／脚本／POST／Cookie旅程再加生成延迟DOM的真实getBookInfo，比较完整ReturnData／静态JSON及已有明确默认时钟窗口。不读取真实起点正文，不把生成HTML当成真实认证。
- `utf8-characterization`：保留六个生成UTF-8观测，使用已有诊断模式。历史截断与Camoufox正确值即使符合已知差异也必须非零退出、overallAccepted=false；不得用continue-on-error、预期失败绿色包装或诊断模式宣布严格兼容。

测试结果从本次实际运行读取；入口和生成元数据单测通过不等于三方已执行。更不能把一次生成metadata通过扩大为全部原始JAR／真实书源／登录／正文差分通过。

## 本地检查与已验收的来源

初版13个生成守卫、本机Python395项／1平台跳过、发布安全Node82和来源API检查通过；不代表业务三方已执行。实际来源Native37783548430/source2f8b37aa/tested ba1817b4自己的四普通工作流与八设置图已独立验收，见[展示修复](HTTP-TTS-DISPLAY-2026-10-08.md#展示增量自己的托管实跑已完成)。

## 初版专项已执行，原件身份失败

入口源码 `3ab4548de55b1ba47056a78ec521e3746903c179`，实际[metadata专项37790757361](https://github.com/warpdotsys/reader-dev/actions/runs/37790757361)／作业113357213174失败。来源API、v8下载、精确AMD64原归档与JAR／镜像导入通过，但原件身份拒绝：**没有启动原JAR或业务三方，没有观察运行aggregate资源预算，没有继续UTF8诊断，也未上传原件。**保存的五份小JSON只有来源与预载／实际加载身份；没有把缺失的THREE_WAY或PROVENANCE补成成功。

单独[公开身份复核37791961831](https://github.com/warpdotsys/reader-dev/actions/runs/37791961831)再次确认归档：manifest `sha256:910043799b510b5796baf99537507a63ca95995cda7de95cbf2ba5eeb9dcb5d1`，JAR 72,914,622 B／`7222fbd0c55d8b7e6f6bc1bbf5f6637fbee45dfbd65eda0ef59e38b2e24cd43a`；本机原件本次只读复核为72,913,887 B／b26原SHA。这个差异[2026-09-28已记录](ORIGINAL-JAR-PROVENANCE-2026-09-28.md)，本次未先核对旧证据导致不必要的大产物下载，现已纠正，不再把该公开镜像视为“同SHA原件”的可用来源。

同一3ab／tested `fd140e93378c83867dac7aed53791e0be9593d8e`自己的四普通工作流均实际成功，独立下载小回执核验：Vue25真实UI／零跳过、287前端＋20取图守卫及类型构建通过，八设置图已逐张目视；Java Python395全执行、JVM170／31环境跳过，两次干净JAR ce76a5同字节，285,649,123 B／1,562条目；自己的Full和双架构Native同JAR，Cam20／helper3、重载和24publisher JSON通过。Full峰值831,148,032 B／PID196，Native874,291,200 B／PID208；普通短测允许1GiB swap、实际0，不能借作本次未执行的零swap三方。完整身份与失败边界见[逐文件证据](evidence/hosted-three-way-3ab4548d-2026-10-08.json)。

## 身份优先纠正及哈希表示回归

新入口把原件身份核验放在两个native下载步骤之前：来源合法后才创建不启动的精确所有权提取容器，保存实际JAR大小／SHA／镜像摘要，无论匹配与否都做精确清理及剩余标签观察；错版必须非零退出，并跳过大JAR和镜像下载。后续主步骤再次核验同一次preflight文件与原SHA，不重新下载或换基准。

源码346dad61的[fail-fast实跑37794305558](https://github.com/warpdotsys/reader-dev/actions/runs/37794305558)已独立验收：来源步骤成功、原件身份步骤明确失败，两个大产物下载步骤与业务步骤实际skipped；四份JSON仅来源三文件＋ORIGINAL_ARCHIVE_IDENTITY。真实提取不启动／实际大小SHA／精确标签删除后剩余0已记录，没有加载native镜像或启动JAR。官方结论仍failure，`strictCompatibilityAccepted=false`；这只证明安全拒绝顺序，不包装为三方通过。[自己的逐文件SHA与失败证据](evidence/hosted-three-way-fast-fail-346dad61-2026-10-08.json)。

另外现有比较器把哈希记录成大写，新入口原先按小写字面比较。新增回归先14项／1错误／exit1，再修为只比较严格64位十六进制代表的同32字节；不去空白、不接受前缀／缺位／错误值，也不改写任何原始JSON。这是独立的验证器缺陷，**不是本次7222与b26不一致的原因**。现在17项定向、全套Python399／1平台跳过／24.776秒通过。

## Python 3.10 兼容回归

同一346dad61／tested c4ce5165的[普通Full37794141611](https://github.com/warpdotsys/reader-dev/actions/runs/37794141611)和[Native37794141150](https://github.com/warpdotsys/reader-dev/actions/runs/37794141150)实际失败于新生成单测：它第一次调用digest文件路径，Python3.10.22没有 `hashlib.file_digest`，399项／1错误，在构建前停止。后续上传无JVM报告是此前尚未构建的结果，不能写成业务或引擎失败；两个工作流保持失败。Java和Vue成功也不能抵消这些失败。

修正只改新驱动的哈希实现：1MiB分块流式计算相同SHA-256，固定依赖／Python3.10／原件比较／文件大小／身份及业务门禁不变。新增跨1MiB边界生成样本与禁用新API守卫，先18项／1失败／exit1，修正后18全通过；全套Python400／1平台跳过／23.691秒通过。不安装新Python或扩大本机读取范围。修正自身的Python3.10托管结果另验，不借3.12专项或父版绿灯。

GitHub手动入口为已有 `browser-image.yml`，设置 `three_way_native_run=37783548430`、`three_way_source=2f8b37aa1c786d87e84d9ac6b745105aa35972a4`、`three_way_revision=ba1817b4256bfde0b0019ee8861d99af18c037a0`，另选一种模式；其他公网／probe／soak模式必须为空／false。具体代码checkout与被测镜像revision分开记录。先执行metadata，完成后再按新事实决定下一专项，不并发重复跑旧失败网站条件。

## 已知风险与回滚

公开归档必须仍可下载且SHA相同，否则停止，不上传私有原件绕过。Docker必须已经使用systemd cgroups；磁盘不足时拒绝，不清理未拥有镜像或修改daemon。GNU timeout先TERM使原harness按所有权清理，超时／清理观察失败不算成功。原JAR启动／参考引擎兼容、最新样本和托管时序均可能暴露新失败，应保留而不是删用例或放宽预算。

回滚仅不再选择新专项或撤回新入口／驱动／生成守卫；不撤销已接受的前端修复，不改部署或原JAR，不删除用户报告。真实认证、本机精确运行、最终三方全部范围、版本提升和正式发布仍未完成。
