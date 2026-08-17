import { test } from 'node:test'
import assert from 'node:assert/strict'
import { canRescanBook, isLocalUrl, isLocBook, isStoragePath, isFileExtension } from './localBook.ts'
import type { Book } from '@/types'

function book(bookUrl: string, origin = 'loc_book'): Pick<Book, 'bookUrl' | 'origin'> {
  return { bookUrl, origin }
}

test('GAP 78：local:// 双轨书可重扫', () => {
  assert.equal(isLocalUrl('local://books/abc.epub'), true)
  assert.equal(canRescanBook(book('local://books/abc.epub', 'default')), true)
})

test('GAP 78：loc_book 文件书可重扫（origin=loc_book）', () => {
  assert.equal(isLocBook('loc_book'), true)
  assert.equal(canRescanBook(book('https://x/book.txt', 'loc_book')), true)
})

test('GAP 78：storage/ 路径书可重扫', () => {
  assert.equal(isStoragePath('storage/legado/books/a.txt'), true)
  assert.equal(canRescanBook(book('storage/legado/books/a.txt', 'default')), true)
})

test('GAP 78：文件扩展名书可重扫（大小写不敏感）', () => {
  assert.equal(isFileExtension('https://x/a.EPUB'), true)
  assert.equal(canRescanBook(book('https://x/a.TXT', 'default')), true)
})

test('GAP 78：书源书不可重扫（网络书无本地文件可解析）', () => {
  assert.equal(canRescanBook(book('https://www.xx.com/book/123', 'bookSourceA')), false)
  assert.equal(canRescanBook(book('https://x/a.html', 'default')), false)
})
