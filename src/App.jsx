import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Toast from "./components/Toast";

function nowDate() {
  return new Date().toISOString().slice(0, 10);
}

function nowTime() {
  return new Date().toTimeString().slice(0, 5);
}

const appWindow = getCurrentWindow();

export default function App() {
  const [ready, setReady] = useState(false);
  const [diaryDir, setDiaryDir] = useState("");
  const [date, setDate] = useState(nowDate());
  const [time, setTime] = useState(nowTime());
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [loading, setLoading] = useState(false);
  const [toast, setToast] = useState({ type: "success", message: "" });
  const toastTimerRef = useRef(null);

  const canSubmit = useMemo(
    () => content.trim().length > 0 && !loading,
    [content, loading]
  );

  const showToast = useCallback((type, message) => {
    setToast({ type, message });
    if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    toastTimerRef.current = setTimeout(() => {
      setToast({ type: "success", message: "" });
      toastTimerRef.current = null;
    }, 2500);
  }, []);

  useEffect(() => {
    return () => {
      if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    };
  }, []);

  async function loadConfig() {
    try {
      const result = await invoke("read_config");
      setDiaryDir(result.diaryDir);
      setReady(true);
    } catch (err) {
      setReady(false);
    }
  }

  useEffect(() => {
    void loadConfig();
  }, []);

  async function onPickEnv() {
    setLoading(true);
    try {
      const result = await invoke("select_env_file");
      setDiaryDir(result.diaryDir);
      setReady(true);
      showToast("success", "配置完成");
    } catch (err) {
      showToast("error", String(err));
    } finally {
      setLoading(false);
    }
  }

  async function onWrite() {
    if (!canSubmit) return;
    setLoading(true);
    try {
      await invoke("write_diary", {
        payload: { date, time, title: title.trim(), content },
      });
      showToast("success", "日记已保存 ✓");
      setTitle("");
      setContent("");
      setDate(nowDate());
      setTime(nowTime());
    } catch (err) {
      showToast("error", String(err));
    } finally {
      setLoading(false);
    }
  }

  function handleTitlebarDoubleClick(e) {
    if (e.detail === 2) {
      appWindow.toggleMaximize();
    }
  }

  return (
    <div className="container">
      <div
        className="titlebar"
        data-tauri-drag-region
        onClick={handleTitlebarDoubleClick}
      >
        <div className="titlebar-icon" data-tauri-drag-region>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path
              d="M19 3H5a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2V5a2 2 0 00-2-2z"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
            <path
              d="M7 8h10M7 12h6M7 16h8"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
        </div>
        <span className="titlebar-text" data-tauri-drag-region>
          Diary Widget
        </span>
        <div className="window-controls">
          <button
            className="control-btn minimize-btn"
            onClick={() => appWindow.minimize()}
            title="最小化"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="2" y="5.5" width="8" height="1" fill="currentColor" />
            </svg>
          </button>
          <button
            className="control-btn close-btn"
            onClick={() => appWindow.close()}
            title="关闭"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path
                d="M3 3l6 6M9 3l-6 6"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
              />
            </svg>
          </button>
        </div>
      </div>

      <div className="content-area">
        {!ready ? (
          <div className="setup-card">
            <div className="setup-icon">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
                <path
                  d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </div>
            <h2>初始配置</h2>
            <p>选择 MCP 项目的 <code>.env</code> 文件以读取 <code>RAG_BASE_PATH</code></p>
            <button
              type="button"
              className="btn btn-primary"
              disabled={loading}
              onClick={onPickEnv}
            >
              {loading ? (
                <span className="loading-spinner" />
              ) : (
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <path
                    d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              )}
              {loading ? "处理中..." : "选择 .env 文件"}
            </button>
          </div>
        ) : (
          <div className="editor">
            <div className="diary-path">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                <path
                  d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                />
              </svg>
              <span>{diaryDir}</span>
            </div>

            <div className="field-row">
              <label className="field">
                <span className="field-label">日期</span>
                <input
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                />
              </label>
              <label className="field">
                <span className="field-label">时间</span>
                <input
                  type="time"
                  value={time}
                  onChange={(e) => setTime(e.target.value)}
                />
              </label>
            </div>

            <label className="field">
              <span className="field-label">
                标题 <span className="optional-tag">可选</span>
              </span>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="为这篇日记起个标题..."
              />
            </label>

            <label className="field field-grow">
              <span className="field-label">内容</span>
              <textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                placeholder="今天发生了什么..."
              />
            </label>

            <button
              type="button"
              className="btn btn-primary btn-submit"
              onClick={onWrite}
              disabled={!canSubmit}
            >
              {loading ? (
                <span className="loading-spinner" />
              ) : (
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                  <path
                    d="M5 12h14M12 5l7 7-7 7"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              )}
              {loading ? "写入中..." : "保存日记"}
            </button>
          </div>
        )}
      </div>

      <Toast type={toast.type} message={toast.message} />
    </div>
  );
}
