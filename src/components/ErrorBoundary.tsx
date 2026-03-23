import React, { Component, ErrorInfo, ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught an error:", error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            height: "100vh",
            width: "100vw",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            backgroundColor: "#fafafa",
            padding: "20px",
          }}
        >
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: "16px",
              maxWidth: "600px",
            }}
          >
            <h1 style={{ color: "#dc2626", fontSize: "20px", fontWeight: "bold" }}>
              应用错误
            </h1>
            <p style={{ color: "#6b7280", textAlign: "center" }}>
              {this.state.error?.message || "发生了未知错误"}
            </p>
            <details
              style={{
                width: "100%",
                padding: "12px",
                backgroundColor: "#f3f4f6",
                borderRadius: "6px",
                fontSize: "12px",
                color: "#374151",
              }}
            >
              <summary style={{ cursor: "pointer", marginBottom: "8px" }}>
                错误详情
              </summary>
              <pre
                style={{
                  whiteSpace: "pre-wrap",
                  wordBreak: "break-word",
                  overflow: "auto",
                  maxHeight: "200px",
                }}
              >
                {this.state.error?.stack || String(this.state.error)}
              </pre>
            </details>
            <button
              onClick={() => {
                this.setState({ hasError: false, error: null });
                window.location.reload();
              }}
              style={{
                padding: "10px 20px",
                backgroundColor: "#8b5cf6",
                color: "white",
                border: "none",
                borderRadius: "6px",
                cursor: "pointer",
                fontSize: "14px",
              }}
            >
              重新加载应用
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
