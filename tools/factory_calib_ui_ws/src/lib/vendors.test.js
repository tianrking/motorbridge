import { describe, expect, it } from 'vitest';
import {
  buildProbePayload,
  buildScanPayloadExtras,
  buildSetIdPayload,
  getVendorScanDefaults,
} from './vendors';

describe('vendors helpers', () => {
  it('builds damiao scan payload extras', () => {
    expect(buildScanPayloadExtras('damiao', { feedbackBase: '0x12' })).toEqual({
      feedback_base: 0x12,
    });
  });

  it('builds robstride scan payload extras', () => {
    expect(buildScanPayloadExtras('robstride', { feedbackId: '0xFD' })).toEqual({
      feedback_ids: [0xfd],
    });
  });

  it('computes vendor scan defaults', () => {
    expect(getVendorScanDefaults('damiao', { feedbackBase: '0x10' }, 0x03)).toBe(0x13);
    expect(getVendorScanDefaults('robstride', { feedbackId: '0xFE' }, 0x03)).toBe(0xfe);
  });

  it('builds set-id payload', () => {
    const h = { esc_id: 0x01, mst_id: 0x11 };
    expect(buildSetIdPayload('damiao', h, 0x02, 0x12)?.new_feedback_id).toBe(0x12);
    expect(buildSetIdPayload('robstride', h, 0x02, 0x12)?.feedback_id).toBe(0x11);
  });

  it('builds probe payload', () => {
    expect(buildProbePayload('damiao', 0x02, 0x12)).toMatchObject({ feedback_base: 0x10 });
    expect(buildProbePayload('robstride', 0x02, 0xfd)).toMatchObject({ feedback_ids: [0xfd] });
  });
});
