import React from "react";

const DropdownMenu = () => {
  return (
    <div
      className="z-50 absolute top-12 right-0 text-base list-none bg-base divide-y divide-gray-100 rounded-lg shadow"
      id="user-dropdown"
    >
      <div className="px-4 py-3">
        <span className="block text-sm text-titleColor font-poppins">Name Surname</span>
        <span className="block text-sm text-sibarFontPrimaryColor truncate font-poppins">
          email@test.com
        </span>
      </div>
      <ul className="py-2" aria-labelledby="user-menu-button">
        <li>
          <a
            href="#"
            className="block px-4 py-2 text-sm font-poppins text-sibarFontPrimaryColor hover:bg-sidebarBtnHoover "
          >
            Settings
          </a>
        </li>
        <li>
          <a
            href="#"
            className="block px-4 py-2 text-sm font-poppins text-sibarFontPrimaryColor hover:bg-sidebarBtnHoover "
          >
            Sign out
          </a>
        </li>
      </ul>
    </div>
  );
};

export default DropdownMenu;
