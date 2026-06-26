import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { FastForwardControl } from './FastForwardControl';

describe('FastForwardControl', () => {
  it('renders input and set button', () => {
    render(<FastForwardControl />);
    expect(screen.getByRole('spinbutton')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /set/i })).toBeInTheDocument();
  });

  it('calls onTimeScaleChange with parsed value on button click', () => {
    const onChange = vi.fn();
    render(<FastForwardControl onTimeScaleChange={onChange} />);
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '20' } });
    fireEvent.click(screen.getByRole('button', { name: /set/i }));
    expect(onChange).toHaveBeenCalledWith(20);
  });

  it('calls onTimeScaleChange on Enter key', () => {
    const onChange = vi.fn();
    render(<FastForwardControl onTimeScaleChange={onChange} />);
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '50' } });
    fireEvent.keyDown(screen.getByRole('spinbutton'), { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith(50);
  });

  it('ignores zero', () => {
    const onChange = vi.fn();
    render(<FastForwardControl onTimeScaleChange={onChange} />);
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '0' } });
    fireEvent.click(screen.getByRole('button', { name: /set/i }));
    expect(onChange).not.toHaveBeenCalled();
  });

  it('ignores negative values', () => {
    const onChange = vi.fn();
    render(<FastForwardControl onTimeScaleChange={onChange} />);
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '-5' } });
    fireEvent.click(screen.getByRole('button', { name: /set/i }));
    expect(onChange).not.toHaveBeenCalled();
  });

  it('ignores non-numeric input', () => {
    const onChange = vi.fn();
    render(<FastForwardControl onTimeScaleChange={onChange} />);
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: 'abc' } });
    fireEvent.click(screen.getByRole('button', { name: /set/i }));
    expect(onChange).not.toHaveBeenCalled();
  });

  it('is disabled when disabled prop is true', () => {
    render(<FastForwardControl disabled />);
    expect(screen.getByRole('spinbutton')).toBeDisabled();
    expect(screen.getByRole('button', { name: /set/i })).toBeDisabled();
  });

  it('allows fractional values (slow-motion)', () => {
    const onChange = vi.fn();
    render(<FastForwardControl onTimeScaleChange={onChange} />);
    fireEvent.change(screen.getByRole('spinbutton'), { target: { value: '0.5' } });
    fireEvent.click(screen.getByRole('button', { name: /set/i }));
    expect(onChange).toHaveBeenCalledWith(0.5);
  });
});
