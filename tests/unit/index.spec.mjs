import { describe, it, expect } from 'vitest';
import { helloWorld } from '../../index.js';
import os from 'node:os';

describe('ネイティブモジュールのテスト', () => {
  it('helloWorld関数がRustからの文字列を返す', () => {
    const result = helloWorld();
    expect(result).toContain("Rust");

    // OSごとに異なるメッセージを返すことを検証
    const platform = os.platform();
    if (platform === "win32") {
      expect(result).toContain("Windows");
    } else if (platform === "darwin") {
      expect(result).toContain("macOS");
    } else if (platform === "linux") {
      expect(result).toContain("Linux");
    }
  });
});
