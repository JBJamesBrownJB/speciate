import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { RenderPipelinePanel } from './RenderPipelinePanel';
import type { RenderPipelineMetrics } from '../types';

const sample: RenderPipelineMetrics = {
  distinctGapMeanMs: 50,
  distinctGapStdMs: 16,
  distinctGapMinMs: 27,
  distinctGapMaxMs: 68,
  deliveryMeanMs: 32,
  alphaResetMean: 0.84,
  alphaResetMin: 0.6,
  alphaResetMax: 1.0,
  stallFrames: 22,
  totalFrames: 100,
  distinctCount: 20,
  duplicateCount: 12,
};

describe('RenderPipelinePanel', () => {
  it('shows a waiting state before any metrics arrive', () => {
    render(<RenderPipelinePanel metrics={undefined} />);
    expect(screen.getByText(/waiting for render-pipeline metrics/i)).toBeInTheDocument();
  });

  it('renders each metric with a label, a value, and a self-documenting tooltip', () => {
    const { container } = render(<RenderPipelinePanel metrics={sample} />);

    // Labels present (exact, to avoid matching the sparkline captions)
    expect(screen.getByText('Snapshot gap')).toBeInTheDocument();
    expect(screen.getByText('Lerp completion (α@reset)')).toBeInTheDocument();
    expect(screen.getByText('Stall frames')).toBeInTheDocument();
    expect(screen.getByText('Snapshot rate')).toBeInTheDocument();

    // Values rendered
    expect(screen.getByText(/50 ms · σ16 \(27–68\)/)).toBeInTheDocument();
    expect(screen.getByText('0.84 (0.60–1.00)')).toBeInTheDocument();

    // Six metric rows, each with a learn-what-it-measures tooltip.
    expect(container.querySelectorAll('.render-metric-row')).toHaveLength(6);
    expect(container.querySelectorAll('.rm-tooltip')).toHaveLength(6);
    expect(screen.getAllByText('Measures:', { selector: 'strong' })).toHaveLength(6);
    expect(screen.getAllByText('Healthy:', { selector: 'strong' })).toHaveLength(6);
    expect(screen.getAllByText('Jitter bug:', { selector: 'strong' })).toHaveLength(6);
  });
});
