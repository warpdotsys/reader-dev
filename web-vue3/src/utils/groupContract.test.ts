import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
  decodeGroupMask,
  encodeGroupMask,
  fromLegacyBookGroup,
  isInLegacyBookGroup,
  toLegacyBookGroup,
  toLegacyBookGroupOrder,
} from './groupContract.ts'

test('legacy group response is normalized to the Vue 3 view model', () => {
  assert.deepEqual(
    fromLegacyBookGroup({ groupId: 8, groupName: '稍后读', order: 3, show: false, cover: '/cover.png' }),
    { groupId: 8, groupName: '稍后读', order: 3, show: false, cover: '/cover.png', id: 8, name: '稍后读', bookCount: undefined },
  )
})

test('group save and order payloads use the Java/Kotlin field names', () => {
  assert.deepEqual(toLegacyBookGroup({ id: 8, name: '待读', orderNum: 2, cover: null, show: true }), {
    groupId: 8,
    groupName: '待读',
    order: 2,
    cover: null,
    show: true,
  })
  assert.deepEqual(toLegacyBookGroupOrder([{ id: 8, orderNum: 0 }, { id: 16, orderNum: 1 }]), [
    { groupId: 8, order: 0 },
    { groupId: 16, order: 1 },
  ])
})

test('multi-group legacy masks round-trip without 32-bit truncation', () => {
  const ids = [1, 4, 1_048_576]
  const mask = encodeGroupMask(ids)
  assert.equal(mask, 1_048_581)
  assert.deepEqual(decodeGroupMask(mask), ids)
  assert.deepEqual(decodeGroupMask(0), [])
})

test('refuses invalid IDs and masks instead of silently corrupting group membership', () => {
  assert.throws(() => encodeGroupMask([3]), /单个二进制位/)
  assert.throws(() => encodeGroupMask([0]), /无效/)
  assert.throws(() => decodeGroupMask(9_007_199_254_740_993), /安全处理/)
})

test('legacy built-in shelf filters keep their original meaning', () => {
  assert.equal(isInLegacyBookGroup({ origin: 'loc_book' }, -2), true)
  assert.equal(isInLegacyBookGroup({ type: 1 }, -3), true)
  assert.equal(isInLegacyBookGroup({ group: 0 }, -4), true)
  assert.equal(isInLegacyBookGroup({ canUpdate: true, lastCheckError: 'timeout' }, -5), true)
  assert.equal(isInLegacyBookGroup({ group: 3 }, 1), true)
  assert.equal(isInLegacyBookGroup({ group: 3 }, 2), true)
  assert.equal(isInLegacyBookGroup({ group: 3 }, 4), false)
  assert.equal(isInLegacyBookGroup({ group: 0 }, -1), true)
})
