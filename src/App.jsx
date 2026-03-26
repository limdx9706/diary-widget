import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Toast from "./components/Toast";

function nowDate() {
  return new Date().toISOString().slice(0, 10);
}

function nowTime() {
  return new Date().toTimeString().slice(0, 5);
}

export default function App() {
  const [ready, setReady] = useState(false);
  const [diaryDir, setDiaryDir] = useState("");
  const [date, setDate] = useState(nowDate());
  const [time, setTime] = useState(nowTime());
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [loading, setLoading] = useState(false);
  const [toast, setToast] = useState({ type: "success", message: "" });

  const canSubmit = useMemo(() => content.trim().length > 0 && !loading, [content, loading]);

  function showToast(type, message) {
    setToast({ type, message });
    window.clearTimeout(showToast.timerId);
    showToast.timerId = window.setTimeout(() => {
      setToast({ type: "success", message: "" });
    }, 2000);
  }

  async function loadConfig() {
    try {
      const result = await invoke("read_config");
      setDiaryDir(result.diaryDir);
      setReady(true);
    } catch (err) {
      setReady(false);
      showToast("error", String(err));
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
      showToast("success", "已完成配置");
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
      const result = await invoke("write_diary", {
        payload: { date, time, title: title.trim(), content }
      });
      showToast("success", `写入成功: ${result.path}`);
      setDate(nowDate());
      setTime(nowTime());
    } catch (err) {
      showToast("error", String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="container">
      <div className="titlebar" data-tauri-drag-region>
        <div className="title" data-tauri-drag-region>Diary Widget</div>
        <div className="window-actions">
          <button type="button" onClick={() => getCurrentWindow().minimize()}>-</button>
          <button type="button" onClick={() => getCurrentWindow().close()}>x</button>
        </div>
      </div>

      {!ready ? (
        <div className="setup-card">
          <h2>先完成配置</h2>
          <p>请选择 MCP 项目的 .env 文件，以读取 RAG_BASE_PATH。</p>
          <button type="button" disabled={loading} onClick={onPickEnv}>
            {loading ? "处理中..." : "选择 .env 文件"}
          </button>
        </div>
      ) : (
        <div className="editor">
          <p className="path">日记目录: {diaryDir}</p>
          <div className="row">
            <label>
              日期
              <input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
            </label>
            <label>
              时间
              <input type="time" value={time} onChange={(e) => setTime(e.target.value)} />
            </label>
          </div>

          <label>
            标题(可选)
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="可留空"
            />
          </label>

          <label>
            内容
            <textarea
              rows={10}
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="写点什么..."
            />
          </label>

          <button type="button" onClick={onWrite} disabled={!canSubmit}>
            {loading ? "写入中..." : "写入日记"}
          </button>
        </div>
      )}
      <Toast type={toast.type} message={toast.message} />
    </div>
  );
}
