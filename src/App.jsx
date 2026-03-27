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

function MinimizeIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M5 12h14" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
      <path d="M18 6L6 18M6 6l12 12" />
    </svg>
  );
}

function FolderIcon() {
  return (
    <svg className="setup-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
    </svg>
  );
}

export default function App() {
  const [ready, setReady] = useState(false);
  const [diaryDir, setDiaryDir] = useState("");
  const [date, setDate] = useState(nowDate());
  const [time, setTime] = useState(nowTime());
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [loading, setLoading] = useState(false);
  const [toast, setToast] = useState({ type: "success", message: "", visible: false });
  const toastTimerRef = useRef(null);
  const textareaRef = useRef(null);

  const canSubmit = useMemo(
    () => content.trim().length > 0 && !loading,
    [content, loading]
  );

  const showToast = useCallback((type, message) => {
    if (toastTimerRef.current) clearTimeout(toastTimerRef.current);
    setToast({ type, message, visible: true });
    toastTimerRef.current = setTimeout(() => {
      setToast((prev) => ({ ...prev, visible: false }));
    }, 2500);
  }, []);

  async function loadConfig() {
    try {
      const result = await invoke("read_config");
      setDiaryDir(result.diaryDir);
      setReady(true);
    } catch {
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

  const onWrite = useCallback(async () => {
    if (!canSubmit) return;
    setLoading(true);
    try {
      await invoke("write_diary", {
        payload: { date, time, title: title.trim(), content },
      });
      showToast("success", "写入成功");
      setContent("");
      setTitle("");
      setDate(nowDate());
      setTime(nowTime());
      textareaRef.current?.focus();
    } catch (err) {
      showToast("error", String(err));
    } finally {
      setLoading(false);
    }
  }, [canSubmit, date, time, title, content, showToast]);

  useEffect(() => {
    function handleKeyDown(e) {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter" && canSubmit) {
        e.preventDefault();
        onWrite();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [canSubmit, onWrite]);

  const appWindow = getCurrentWindow();

  return (
    <div className="container">
      <div className="titlebar" data-tauri-drag-region>
        <div className="title" data-tauri-drag-region>
          Diary Widget
        </div>
        <div className="window-actions">
          <button type="button" onClick={() => appWindow.minimize()} title="最小化">
            <MinimizeIcon />
          </button>
          <button type="button" className="btn-close" onClick={() => appWindow.close()} title="关闭">
            <CloseIcon />
          </button>
        </div>
      </div>

      {!ready ? (
        <div className="setup-card">
          <FolderIcon />
          <h2>欢迎使用</h2>
          <p>请选择 MCP 项目的 .env 文件，以读取 RAG_BASE_PATH 配置日记存储路径。</p>
          <button
            type="button"
            className="btn-primary"
            disabled={loading}
            onClick={onPickEnv}
          >
            {loading ? (
              <>
                <span className="spinner" />
                处理中...
              </>
            ) : (
              "选择 .env 文件"
            )}
          </button>
        </div>
      ) : (
        <div className="editor">
          <p className="path">{diaryDir}</p>
          <div className="row">
            <label>
              日期
              <input
                type="date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
              />
            </label>
            <label>
              时间
              <input
                type="time"
                value={time}
                onChange={(e) => setTime(e.target.value)}
              />
            </label>
          </div>

          <label>
            标题（可选）
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="给这条日记起个名字…"
            />
          </label>

          <label>
            内容
            <div className="textarea-wrapper">
              <textarea
                ref={textareaRef}
                rows={8}
                value={content}
                onChange={(e) => setContent(e.target.value)}
                placeholder="写点什么…"
              />
              <span className="char-count">{content.length} 字</span>
            </div>
          </label>

          <div className="editor-footer">
            <span className="kbd-hint">
              <kbd>Ctrl</kbd> + <kbd>Enter</kbd> 提交
            </span>
            <button
              type="button"
              className="btn-primary"
              onClick={onWrite}
              disabled={!canSubmit}
            >
              {loading ? (
                <>
                  <span className="spinner" />
                  写入中...
                </>
              ) : (
                "写入日记"
              )}
            </button>
          </div>
        </div>
      )}
      <Toast type={toast.type} message={toast.message} visible={toast.visible} />
    </div>
  );
}
