# 纯生成大 EPUB 整镜像回归

只生成公开固定常量，不读取书、正文、Cookie 或宿主浏览器。850 章／1690 条目，压缩36–40 MiB、展开88–92 MiB；所有条目仍受现有64／256／32／4 MiB及8192条限制。下载60秒、Worker创建起15秒、界面观察30秒不变。小样本单元测试不算整镜像验收。

需要Linux、systemd cgroup v2和systemd驱动的Docker。已校验的完整镜像必须默认UID10001、Vue3／Camoufox、原JVM256／768，传入准确的镜像构建提交及包内JAR摘要。脚本不拉取、重建、替换镜像或接入生产数据。

```sh
sudo python3 scripts/epub-pressure/runner.py \
  --image '<verified immutable image id>' --architecture amd64 \
  --revision '<40-character image build revision>' \
  --jar-sha '<64-character packaged JAR SHA-256>' \
  --work-root '<existing authorized temporary root>' \
  --report '<new report path>'
python3 scripts/epub-pressure/verify.py '<new report path>' \
  --architecture amd64 --revision '<same image build revision>' \
  --jar-sha '<same packaged JAR SHA-256>'
```

三个全新进程／存储串行执行；一次失败保留并停止，不用后续成功覆盖。控制器、生成器、Java、随镜像提供的实际Camoufox客户端及子进程共用CPU2／硬2GiB／high1.5GiB／swap0／父PID512、叶256，网络只有lo、不公开端口。生成账号由实际Reader接口写入，不伪造users.json或密码哈希。登录、错误密码、退出／Token撤销、记住登录／刷新／新标签隔离、八个阅读场景和805／806标题／严格正文回归保留。测试后只清自己的生成数据、容器与空父组，保留无正文收据；失败清理也核验准确标签／镜像／挂载。

此处“全新”只指每轮进程、浏览器上下文及数据：不清空宿主全局page cache，不声称整个操作系统冷态、长期稳定、原JAR数据兼容或真实站点认证已经通过。生成账号不是原B26 JAR生成账号；原件对照仍由已有独立实测提供。不要将单元测试、校验器的合成收据或ZIP完整性检查当作实际界面通过。

2026-10-14：9项生成器、12项验收校验器和5项资源观察器单元测试通过；128项发布流程静态回归通过。实际38,626,835B／93,511,402B展开样本的CRC／XML／目录／摘要通过，该D盘临时ZIP已删除。验收器拒绝少轮／少场景、错身份／源码、客户端预算外、放宽期限／资源、OOM、未清理、章节错配和把合成账号提升为真实认证等结果。

Native托管流程已接入原生构建／新runner重载各自三轮的实际执行和独立验收，两架构均为必要门禁；失败保留独立小报告，不改变发布器既有32份收据结构。新的整镜像运行结果尚待验证，不能用上述单元或静态检查充当通过。隔离服务器准备连接失败不算业务执行或拒绝；未经实际整镜像结果和独立校验，不作为发版依据。
