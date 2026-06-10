type MetricProps = {
  label: string;
  value: string | number;
  hint: string;
};

export function Metric({ label, value, hint }: MetricProps) {
  return (
    <article className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{hint}</small>
    </article>
  );
}
