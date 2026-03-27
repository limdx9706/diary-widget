import { useEffect, useState } from "react";

function SuccessIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M20 6L9 17l-5-5" />
    </svg>
  );
}

function ErrorIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <path d="M12 8v4M12 16h.01" />
    </svg>
  );
}

export default function Toast({ type, message, visible }) {
  const [show, setShow] = useState(false);
  const [hiding, setHiding] = useState(false);

  useEffect(() => {
    if (visible && message) {
      setHiding(false);
      setShow(true);
    } else if (!visible && show) {
      setHiding(true);
      const timer = setTimeout(() => {
        setShow(false);
        setHiding(false);
      }, 250);
      return () => clearTimeout(timer);
    }
  }, [visible, message]);

  if (!show || !message) return null;

  return (
    <div className={`toast ${type} ${hiding ? "hiding" : ""}`}>
      {type === "success" ? <SuccessIcon /> : <ErrorIcon />}
      {message}
    </div>
  );
}
