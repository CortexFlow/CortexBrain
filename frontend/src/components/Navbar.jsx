import React, { useState } from "react";
import ProfileImg from "../assets/img/profile-img.svg";
import { FaBell } from "react-icons/fa"; // Usa solo react-icons
import DropdownMenu from "./DropdownMenu";

const Navbar = () => {
  // Stato per controllare l'apertura/chiusura del dropdown
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);

  // open and close the dropdown menu
  const toggleDropdown = () => {
    setIsDropdownOpen(!isDropdownOpen);
  };

  return (
    <nav className="bg-base border-gray-200 relative">
      {/* Posizione relativa per il contesto */}
      <div className="flex flex-wrap items-center justify-between mx-auto p-4">
        <div className="font-poppins font-medium text-[16px] text-titleColor">
          Hello [User]
        </div>

        <div className="flex ml-auto items-center gap-6">

          {/* Bell icon */}
          <div className="flex items-center justify-center w-8 h-8 text-xl text-titleColor">
            <FaBell /> {/* Usa direttamente l'icona FaBell da react-icons */}
          </div>

          {/* Icona utente */}
          <button
            type="button"
            className="flex items-center justify-center text-sm bg-base rounded-full focus:ring-4 focus:ring-gray-300"
            id="user-menu-button"
            aria-expanded={isDropdownOpen ? "true" : "false"}
            onClick={toggleDropdown}
          >
            <span className="sr-only">Open user menu</span>
            <img
              className="w-8 h-8 rounded-full"
              src={ProfileImg}
              alt="user photo"
            />
          </button>
        </div>
      </div>

      {/* Dropdown menu */}
      {isDropdownOpen && <DropdownMenu />}

      <button
        data-collapse-toggle="navbar-user"
        type="button"
        className="inline-flex items-center p-2 w-10 h-10 justify-center text-sm text-sibarFontPrimaryColor 
        rounded-lg md:hidden hover:bg-base focus:outline-none focus:ring-2 focus:ring-gray-200"
        aria-controls="navbar-user"
        aria-expanded="false"
      >
        <span className="sr-only">Open main menu</span>
        <svg
          className="w-5 h-5"
          aria-hidden="true"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 17 14"
        >
          <path
            stroke="currentColor"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            d="M1 1h15M1 7h15M1 13h15"
          />
        </svg>
      </button>
    </nav>
  );
};

export default Navbar;
