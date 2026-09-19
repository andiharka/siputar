import { afterAll, beforeEach, expect, test } from 'bun:test';
import { warmupAudioDevice } from '../src/lib/utils/audioWarmup.ts';

const originalWindow = globalThis.window;
let starts = 0;
let resumes = 0;
let resumeImpl = async () => {};
const context = {
  state: 'suspended', sampleRate: 44100,
  resume: () => { resumes++; return resumeImpl(); },
  createBuffer: () => ({}), destination: {},
  createBufferSource: () => ({ connect() {}, disconnect() {}, start() { starts++; } }),
};
globalThis.window = { AudioContext: class { constructor() { return context; } } };
afterAll(() => { globalThis.window = originalWindow; });
beforeEach(() => { starts = 0; resumes = 0; context.state = 'suspended'; });

test('a suspended audio context cannot indefinitely block scheduled playback', async () => {
  resumeImpl = () => new Promise(() => {});
  await warmupAudioDevice();
  expect(resumes).toBe(1);
  expect(starts).toBe(0);
}, 1500);

test('warm-up can recover after an earlier resume timed out', async () => {
  resumeImpl = async () => { context.state = 'running'; };
  await warmupAudioDevice();
  expect(starts).toBe(1);
});

test('concurrent readiness and playback requests share a single warm-up', async () => {
  resumeImpl = async () => { context.state = 'running'; };
  await Promise.all([warmupAudioDevice(), warmupAudioDevice(), warmupAudioDevice()]);
  expect(resumes).toBe(1);
  expect(starts).toBe(1);
});
