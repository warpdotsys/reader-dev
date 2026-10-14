# v6.1.0 准确双架构镜像：30 分钟生成数据长测

## 已从受测产物验证

源码 `678a0d1cba8a92f1963127f81e0908d44e523201`，合并测试快照 `ffa69b73d13565b391cc9a079df3d9a2a8373105`，源 Native 运行 `38496598950`。这次使用该 Native 已构建、已原生运行及重新导入验收的两份原归档，不在长测中重建。共同 JAR 为 `1f114993a273eb2d5dc30d47ff3b10446ce30fccd04bb6a49e46f42cef2138c5`、版本 6.1.0。产品源码相对 f6 未变，但这两份镜像身份不同，不能借用 f6/c76 服务器单书或公开书成果。

| GitHub 托管运行 | 连续实际请求 | 轮次／成功搜索 | 叶内存峰值 B | PID 峰值 |
| --- | --- | --- | --- | --- |
| [AMD64 38500836557](https://github.com/warpdotsys/reader-dev/actions/runs/38500836557) | 1807.881 秒 | 153／620 | 1,004,945,408 | 211 |
| [ARM64 38500839954](https://github.com/warpdotsys/reader-dev/actions/runs/38500839954) | 1803.349 秒 | 112／456 | 967,868,416 | 201 |

AMD64 镜像配置 ID 为 `sha256:ad53b6918e1f7e3af1fac77c7c76aef66df9f3b417c037bbeda1cf1edd148ba0`；ARM64 为 `sha256:8750ba600b885bc97dde49e45033b9a2136251f7c216b6e0e060f45b28230fb5`。加载前后、运行 JAR、合并快照、源 Native、容器无外网均逐字段核验。每次仅一项长测作业执行，其余互斥模式跳过，不将跳过记为覆盖；分别运行在 GitHub 的 ubuntu-24.04、ubuntu-24.04-arm，没有自托管 runner。

## 独立验收内容

四个生成账号并发，每轮交替 GET／POST；全部 265 行原始连续轨迹重新核对，尾轮必须达到 1800 秒，不能用请求结束后的空等补时。目标 Cookie／请求形态错误为零。每侧实际验证导航超时、脚本 watchdog、自有 worker 异常退出的失败 ReturnData 和随后健康恢复；冷、暖、每次恢复、每轮及最终静止时 worker／browser／driver 均为零。

CPU 2、硬内存 2 GiB、父级 high 1.5 GiB、零 swap、PID 256 的原预算未放宽。父级峰值分别 1,004,961,792／967,884,800 B，父级和叶级 high／硬触限／OOM／OOM-kill／PID 触限均为零。删除前容器健康；删除后宿主清单中自有容器为零，父级无残留进程、测试单元停用，均由单独收据证明，不以测试脚本正常退出替代清理验收。

只下载 12,325／11,424 B 的两份小报告，共 23,749 B；平台 ZIP digest 和每份解出的原始观察均重新核对。未再次下载 JAR 或约 2 GiB 镜像到 C 盘。第一次独立消费在读取 GitHub jobs API 时失败退出，未生成接受回执；随后仅重试读取，没有重启两条已完成的长测。

完整身份、故障计时、原始报告摘要与独立消费者结果见[机器证据](evidence/candidate-678a0d1c-native-soak-2026-10-14.json)。可对解出的原报告重验：

```powershell
python scripts/verify-bundled-browser-soak.py <AMD64报告目录> --architecture amd64 --expected-revision ffa69b73d13565b391cc9a079df3d9a2a8373105 --expected-source-revision 678a0d1cba8a92f1963127f81e0908d44e523201 --expected-native-run 38496598950 --expected-jar 1f114993a273eb2d5dc30d47ff3b10446ce30fccd04bb6a49e46f42cef2138c5 --expected-image sha256:ad53b6918e1f7e3af1fac77c7c76aef66df9f3b417c037bbeda1cf1edd148ba0 --seconds 1800 --memory-policy high-1536m --require-container-state --require-removal
```

## 已知限制与回滚

首个连续样本到最终的静止内存仍增长 64,106,496／69,550,080 B；最长实际请求 15.630／21.227 秒。保留原值，不将其归因为泄漏或某个缓存，也不据有限半小时样本宣称长期无泄漏、生产容量或真实站点认证。

当前 f6/c76 准确整镜像的获准单书仍[六／八失败](AUTHORIZED-EPUB-REJECTION-2026-10-14.md)：原始 EPUB 刷新触发生产 60 秒下载超时。本次生成网页长测不能覆盖该失败，也不能替代起点登录、全部原版数据兼容或最终上线。PR56 保持草稿，未合并、打标签、正式发布或部署。

本轮只消费已完成测试、汇总有限证据和更新已知问题，不改变产品、默认生产资源策略或私网／TLS 守卫。无需回滚生产；停止使用本诊断流程即可，原 JAR、旧准确软件和全部失败记录保留。58 份用户报告只核对哈希，不读取或覆盖内容。
