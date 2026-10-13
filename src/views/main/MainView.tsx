const PAGES = ["开始", "记录", "科目", "设置"] as const;

export function MainView() {
  return (
    <div className="main-window">
      <nav className="main-nav">
        <div className="brand">Aside</div>
        {PAGES.map((page, i) => (
          <div key={page} className={i === 0 ? "nav-item on" : "nav-item"}>
            {page}
          </div>
        ))}
      </nav>
      <main className="main-page">
        <h1 className="wk">今天也坐下来了</h1>
      </main>
    </div>
  );
}
