import React from "react";

import Lamp from "../assets/img/lamp.png"

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
      <div className="relative flex items-center justify-center">
        <svg
          width="207"
          height="234"
          viewBox="0 0 207 234"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          className="absolute"
        >
          <path
            fillRule="evenodd"
            clipRule="evenodd"
            d="M136.743 33.6287C136.743 15.4033 121.968 0.628662 103.743 0.628662C85.5173 0.628662 70.7427 15.4033 70.7427 33.6287H16.7427C7.90612 33.6287 0.742676 40.7921 0.742676 49.6287V217.629C0.742676 226.465 7.90612 233.629 16.7427 233.629H190.743C199.579 233.629 206.743 226.465 206.743 217.629V49.6287C206.743 40.7921 199.579 33.6287 190.743 33.6287H136.743Z"
            fill="#D5C4FF"
            fillOpacity="0.4"
          />
        </svg>
        <div className="bg-transparent p-4 rounded-lg mt-8 text-center relative z-10">
          <img
            src={Lamp}
            alt="Advice"
            className="mx-auto mb-3"
          />

          <h2 className="text-sm font-poppins text-AdviceBoxColor">
            CortexFlow Advices
          </h2>
          <p className="text-xs text-AdviceBoxColor mt-2 font-poppins">
            Create your first pipeline by clicking in the "Pipelines" section.
          </p>
        </div>
      </div>
    </div>
  );
};

export default SidebarMenu;
