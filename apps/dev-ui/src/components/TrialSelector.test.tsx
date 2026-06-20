import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TrialSelector } from './TrialSelector';

// Mutable holder so each test can control the generated template list.
const mock = vi.hoisted(() => ({ templates: [] as Array<{ name: string; label: string; description: string }> }));

vi.mock('../generated/trial-templates', () => ({
  get TRIAL_TEMPLATES() {
    return mock.templates;
  },
}));

describe('TrialSelector', () => {
  beforeEach(() => {
    mock.templates = [];
  });

  it('does not throw and shows a fallback when no templates are generated', () => {
    mock.templates = [];
    // Must not throw (previously crashed on TRIAL_TEMPLATES[0].name -> white box).
    expect(() => render(<TrialSelector onLoadTrial={() => {}} />)).not.toThrow();
    expect(screen.getByText(/no trial templates/i)).toBeInTheDocument();
  });

  it('renders the selector with a default selection when templates exist', () => {
    mock.templates = [
      { name: 'alpha', label: 'Alpha', description: 'first' },
      { name: 'beta', label: 'Beta', description: 'second' },
    ];
    render(<TrialSelector onLoadTrial={() => {}} />);
    expect(screen.getByRole('combobox')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /load trial/i })).toBeInTheDocument();
  });
});
