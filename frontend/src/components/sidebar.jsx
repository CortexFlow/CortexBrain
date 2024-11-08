import React from "react";

const SidebarMenu = () => {
  return (
    <div className="sidebar">
      <header className="sidebar-header">
        <h1 className="title">CortexFlow</h1>
        <p className="user-panel">User Panel</p>
      </header>
      <nav className="sidebar-nav">
        <ul>
          <li className="active">Dashboard</li>
          <li>Nodes</li>
          <li>Pipeline</li>
          <li>Config</li>
          <li>Roles</li>
          <li>Settings</li>
          <li>Tutorials</li>
        </ul>
      </nav>
      <div className="advice">
        <img src="https://placeholder.pics/svg/64x38" alt="Advice" />
        <div>
          <h2 className="advice-title">CortexFlow Advices</h2>
          <p className="advice-content">
            Create your first pipeline by clicking in the "Pipelines" section.
          </p>
        </div>
      </div>
      <footer className="sidebar-footer">
        <a href="#logout" className="logout">
          Logout
        </a>
      </footer>
    </div>
  );
};

export default SidebarMenu;
