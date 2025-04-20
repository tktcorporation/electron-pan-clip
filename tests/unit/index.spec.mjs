import { describe, it, expect } from 'vitest';
import { helloWorld } from '../../index.js';

describe('ネイティブモジュールのテスト', () => {
  it('helloWorld関数がRustからの文字列を返す', () => {
    const result = helloWorld();
    expect(result).toBe('Hello from Rust!');
  });
});
