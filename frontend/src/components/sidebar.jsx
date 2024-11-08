import React from "react";

const SidebarMenu = () => {
  return (
    <div className="min-h-screen w-64 bg-base flex flex-col p-6 ">
      {/* Header */}
      <header className="mb-8">
        <h1 className="text-2xl font-poppins tracking-wide text-titleColor font-normal text-[23px]">
          CortexFlow
        </h1>
      </header>

      {/* Navigation */}
      <nav>
        <p className="text-sm text-sibarFontPrimaryColor font-poppins font-medium text-[10px] mb-4 mt-2">
          User Panel
        </p>
        <ul className="space-y-4">
          <li className="hover:bg-sidebarBtnHoover transition rounded-lg p-2">
            <a href="#" className="flex items-center">
              <span
                className="text-[16px] text-sibarFontPrimaryColor font-poppins font-medium
              hover:text-sidebarHoover"
              >
                Dashboard
              </span>
            </a>
          </li>
          <li className="hover:bg-sidebarBtnHoover transition rounded-lg p-2">
            <a href="#" className="flex items-center">
              <span
                className="text-[16px] text-sibarFontPrimaryColor font-poppins font-medium 
              hover:text-sidebarHoover"
              >
                Nodes
              </span>
            </a>
          </li>
          <li className="hover:bg-sidebarBtnHoover transition rounded-lg p-2">
            <a href="#" className="flex items-center">
              <span
                className="text-[16px] text-sibarFontPrimaryColor font-poppins font-medium 
              hover:text-sidebarHoover"
              >
                Pipeline
              </span>
            </a>
          </li>
          <li className="hover:bg-sidebarBtnHoover transition rounded-lg p-2">
            <a href="#" className="flex items-center">
              <span
                className="text-[16px] text-sibarFontPrimaryColor font-poppins font-medium
              hover:text-sidebarHoover"
              >
                Config
              </span>
            </a>
          </li>
          <li className="hover:bg-sidebarBtnHoover transition rounded-lg p-2">
            <a href="#" className="flex items-center">
              <span
                className="text-[16px] text-sibarFontPrimaryColor font-poppins font-medium
              hover:text-sidebarHoover"
              >
                Roles
              </span>
            </a>
          </li>
          <li className="hover:bg-sidebarBtnHoover transition rounded-lg p-2">
            <a href="#" className="flex items-center">
              <span
                className="text-[16px] text-sibarFontPrimaryColor  font-poppins font-medium
              hover:text-sidebarHoover"
              >
                Settings
              </span>
            </a>
          </li>
          <li className="hover:bg-sidebarBtnHoover transition rounded-lg p-2">
            <a href="#" className="flex items-center">
              <span
                className="text-[15px] text-sibarFontPrimaryColor font-poppins font-medium 
              hover:text-sidebarHoover"
              >
                Tutorials
              </span>
            </a>
          </li>
          <li className="rounded-lg p-2">
            <a href="#" className="flex items-center">
              <span
                className="text-[15px] text-sibarFontPrimaryColor font-poppins font-medium 
              hover:text-sidebarRedHoover"
              >
                Logout
              </span>
            </a>
          </li>
        </ul>
      </nav>

      {/* Advice Section */}
      <div className="bg-gray-700 p-4 rounded-lg mt-8 text-center">
        <img
          src="https://placeholder.pics/svg/64x38"
          alt="Advice"
          className="mx-auto mb-3"
        />
        <h2 className="text-sm font-poppins text-indigo-400">
          CortexFlow Advices
        </h2>
        <p className="text-xs text-gray-300 mt-2">
          Create your first pipeline by clicking in the "Pipelines" section.
        </p>
      </div>
    </div>
  );
};

export default SidebarMenu;
