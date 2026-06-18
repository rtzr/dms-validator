import type { ReactNode } from "react";

interface Props {
  text: string;
  children: ReactNode;
  /** Which side of the trigger the bubble appears on. */
  side?: "top" | "bottom" | "left";
}

/** Lightweight hover tooltip. The bubble shows on hover/focus of the trigger. */
export function Tooltip({ text, children, side = "bottom" }: Props) {
  return (
    <span className={`tip tip-${side}`} tabIndex={0}>
      {children}
      <span className="tip-bubble" role="tooltip">
        {text}
      </span>
    </span>
  );
}
