/** 仅限制单次跨线程在途结果，不删资源、不改变原书或解压总量上限。 */
export const EPUB_TRANSFER_BATCH_BYTES = 2 * 1024 * 1024
export const EPUB_TRANSFER_BATCH_ENTRIES = 64

export interface EpubTransferAck { batchAccepted: number }
