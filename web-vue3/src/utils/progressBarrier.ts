/** 等待既有进度写入完成，避免旧 POST 晚到覆盖新章 GET 的进度。 */
export class ProgressWriteBarrier {
  private readonly pending = new Set<Promise<unknown>>()

  track<T>(write: Promise<T>): Promise<T> {
    const tracked = write.finally(() => this.pending.delete(tracked))
    this.pending.add(tracked)
    return tracked
  }

  async settle(): Promise<void> {
    // 只等待开始读取时已经发出的写入；之后的写入使用新的当前章编号。
    // 失败仍交给原调用者处理，不能永远阻塞阅读或伪报保存成功。
    await Promise.all([...this.pending].map(write => write.catch(() => undefined)))
  }
}
