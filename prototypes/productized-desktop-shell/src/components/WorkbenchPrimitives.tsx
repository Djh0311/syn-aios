export function SummaryTile({ label, value, hint }: { label: string; value: string; hint: string }) {
  return (
    <div className="summary-tile">
      <span>{label}</span>
      <strong>{value}</strong>
      <em>{hint}</em>
    </div>
  );
}

export function DetailLine({ label, value, emptyValue }: { label: string; value: string; emptyValue?: string }) {
  const displayValue = emptyValue === undefined ? value : value || emptyValue;
  return (
    <div className="detail-line">
      <span>{label}</span>
      <strong>{displayValue}</strong>
    </div>
  );
}
