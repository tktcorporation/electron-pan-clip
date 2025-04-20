import test from 'ava'

import { helloWorld } from '../../index.js'

test('helloWorld from native', (t) => {
  t.is(helloWorld(), 'Hello from Rust!')
})
