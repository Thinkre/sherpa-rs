// 临时测试组件 - 用于诊断空白页问题
import React from "react";

export const TestPage: React.FC = () => {
  return (
    <div
      style={{
        padding: "20px",
        backgroundColor: "#f0f0f0",
        minHeight: "100vh",
      }}
    >
      <h1 style={{ color: "#333" }}>测试页面</h1>
      <p>如果您能看到这个页面，说明 React 已经正常渲染。</p>
      <p>当前时间: {new Date().toLocaleString()}</p>
      <div
        style={{
          marginTop: "20px",
          padding: "10px",
          backgroundColor: "#fff",
          border: "1px solid #ccc",
        }}
      >
        <h2>调试信息：</h2>
        <ul>
          <li>User Agent: {navigator.userAgent}</li>
          <li>Protocol: {window.location.protocol}</li>
          <li>Host: {window.location.host}</li>
          <li>Pathname: {window.location.pathname}</li>
        </ul>
      </div>
    </div>
  );
};
