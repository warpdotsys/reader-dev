# 正式发布器的配置摘要/manifest摘要桥接

正式 `release-native-artifacts.mjs loaded` 接入上一阶段只读校验工具。归档与共享JAR完整SHA门禁、架构/non-root/原包装入口、准确release/revision、默认Vue3/Camoufox检查先执行。只有加载ID与原配置ID不同才实时调用正式Python工具；缺少ctr、错误字节/层/配置或超时均失败，不按相似环境接受。

原metadata.json及其imageId从不覆写。不同对象的loaded回执在保留原imageId的同时新增loadedImageId与originalConfigIdentity，精确指明manifest实际引用的原配置字节。原同ID路径不调用额外读取器、输出格式保持原样，既有篡改与ID不符测试不删除。布尔true、缺失关联、错误JAR/revision/layers都不能替代真实关联检查。

`run-hosted-three-way.py`区分原制品身份和实际运行ID，拒绝没有准确关联回执的新ID。跨对象时再把原配置摘要/revision/JAR SHA交给三方框架实时重验，并在隔离容器内实读包内JAR；不是只信保存的passed布尔。摘要分别保留原SourceImageId与实际LoadedImageId，不对未知差异做规范化。

## 当前证据

本地Windows的38项发布器/manifest测试：37执行通过、1项Linux桥接集成明确跳过，不能写38全部执行。新Linux测试会通过Node CLI真实调用正式Python校验器，生成命令只提供有哈希的config/manifest/layer数据，并确认篡改失败及metadata不变；它不冒称真实Docker运行。19项托管三方消费者测试通过，完整Python779项/51.465秒、1项既有POSIX跳过、零失败。

7b自己的托管Java与Vue流程已成功并独立接受：JVM47份XML/222项/35既有跳过/零失败，双clean同16f完整JAR；376前端、类型/Vite与25实际Chromium旅程。7b完整镜像及双架构Native仍处验收中，其结果不移借给本次未托管验证的新增发布器源码。

当前服务器可用内存预检不足2GiB，准确整镜像的生成三方b轮未启动任务或创建输出，不降低预检或预算、不结束其他服务。先前原件B26/16f及准确包装镜像均保留只读；本轮不复制或读取任何书籍正文。

## 明确限制与回滚

当前支持原配置ID到containerd未压缩单架构manifest的严格正向关联。原制品本身只记录manifest ID、反向跨存储、压缩层或index的未知映射仍拒绝；尚未宣称支持所有Docker存储组合。新桥接代码自己的托管Linux集成、正式发布器真实跨存储运行、实站认证/原件三方、生产界面和最终发版均仍须对应证据。

回滚可在干净检出撤回发布器及消费者增量；原同ID流程保持可用，不同ID再次严格拒绝。没有推registry、创建release、合并PR、改生产服务、重建原制品或覆写用户文件。保留单书验收与清理、旧身份拒绝/资源预检失败等原证据。
