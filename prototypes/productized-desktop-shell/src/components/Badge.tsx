type BadgeTone = "neutral" | "candidate" | "warning" | "unknown";

type BadgeProps = {
  children: React.ReactNode;
  tone?: BadgeTone;
};

export function Badge({ children, tone = "neutral" }: BadgeProps) {
  return <span className={`badge ${tone}`}>{children}</span>;
}
