export function formatNumber(value: number | undefined | null, decimals = 2): string {
  if (value === undefined || value === null) return '—';
  if (Number.isInteger(value) && Math.abs(value) < 1e6) {
    return value.toLocaleString('en-US');
  }
  return value.toLocaleString('en-US', { maximumFractionDigits: decimals });
}

export function formatScientific(value: number | undefined | null, decimals = 3): string {
  if (value === undefined || value === null) return '—';
  if (value === 0) return '0';
  const exp = Math.floor(Math.log10(Math.abs(value)));
  const mantissa = value / Math.pow(10, exp);
  const mantissaStr = mantissa.toFixed(decimals);
  const superscripts: Record<string, string> = {
    '0': '⁰', '1': '¹', '2': '²', '3': '³', '4': '⁴',
    '5': '⁵', '6': '⁶', '7': '⁷', '8': '⁸', '9': '⁹', '-': '⁻'
  };
  const expStr = String(exp)
    .split('')
    .map(ch => superscripts[ch] ?? ch)
    .join('');
  return `${mantissaStr} × 10${expStr}`;
}

export function formatMeters(value: number | undefined | null): string {
  if (value === undefined || value === null) return '—';
  if (Math.abs(value) >= 1_000) {
    return `${formatNumber(value / 1_000, 1)} km`;
  }
  return `${formatNumber(value, 1)} m`;
}

export function formatKelvin(value: number | undefined | null): string {
  if (value === undefined || value === null) return '—';
  return `${formatNumber(value, 1)} K`;
}

export function formatPressure(value: number | undefined | null): string {
  if (value === undefined || value === null) return '—';
  if (value >= 1_000) {
    return `${formatNumber(value)} Pa`;
  }
  return `${formatNumber(value)} Pa`;
}

export function formatFraction(value: number | undefined | null, decimals = 2): string {
  if (value === undefined || value === null) return '—';
  return `${formatNumber(value * 100, decimals)}%`;
}

export function formatMass(value: number | undefined | null): string {
  if (value === undefined || value === null) return '—';
  if (Math.abs(value) >= 1e24) {
    return `${formatScientific(value, 3)} kg`;
  }
  return `${formatNumber(value)} kg`;
}

export function formatAge(value: number | undefined | null): string {
  if (value === undefined || value === null) return '—';
  if (value >= 1e9) {
    return `${formatScientific(value / 1e9, 2)} Gyr`;
  }
  if (value >= 1e6) {
    return `${formatNumber(value / 1e6, 2)} Myr`;
  }
  return `${formatNumber(value, 0)} s`;
}

export function formatTick(value: number | undefined | null): string {
  if (value === undefined || value === null) return '—';
  return value.toLocaleString('en-US');
}

export function formatIndex(value: number | undefined | null, decimals = 3): string {
  if (value === undefined || value === null) return '—';
  return formatNumber(value, decimals);
}
