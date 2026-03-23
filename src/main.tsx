import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import ErrorBoundary from "./components/ErrorBoundary";

// Initialize i18n
import "./i18n";

// Initialize model store (loads models and sets up event listeners)
import { useModelStore } from "./stores/modelStore";
// Initialize asynchronously to avoid blocking render
useModelStore
  .getState()
  .initialize()
  .catch((err) => {
    console.error("Failed to initialize model store:", err);
  });

// Check if root element exists
const rootElement = document.getElementById("root");
if (!rootElement) {
  console.error("Root element not found!");
  document.body.innerHTML =
    '<div style="padding: 20px; color: red;">错误：找不到根元素</div>';
} else {
  try {
    ReactDOM.createRoot(rootElement).render(
      <React.StrictMode>
        <ErrorBoundary>
          <App />
        </ErrorBoundary>
      </React.StrictMode>,
    );
  } catch (error) {
    console.error("Failed to render app:", error);
    rootElement.innerHTML = `
      <div style="padding: 20px; color: red;">
        <h1>渲染错误</h1>
        <p>${error instanceof Error ? error.message : String(error)}</p>
        <button onclick="window.location.reload()" style="padding: 8px 16px; margin-top: 10px; cursor: pointer;">
          重新加载
        </button>
      </div>
    `;
  }
}
