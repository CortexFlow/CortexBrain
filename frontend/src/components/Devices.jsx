import React from "react";

const Devices = () => {
  return (
    <div className="w-[340px] h-[419px] p-5 bg-white rounded-lg">
      <div className="flex justify-between items-center mb-5">
        <p className="font-poppins font-medium text-[15px] leading-[22.5px]">Devices</p>
        <button className="w-6 h-6 flex justify-center items-center bg-[#6425fe] text-white rounded-[4.36px]">
          +
        </button>
      </div>
      <ul className="list-none p-0">
        {[1, 2, 3, 4, 5].map((device) => (
          <li key={device} className="flex items-center gap-4 mb-4">
            <img
              src="https://placeholder.pics/svg/26x26"
              alt="Device icon"
              className="w-6 h-6"
            />
            <div className="flex flex-col">
              <p className="font-normal text-[15px] leading-[22.5px] text-[#2c2c2c]">
                Device {device}
              </p>
              <p className="font-normal text-[15px] leading-[22.5px] text-[#2c2c2c]">
                Protocol: MQTT
              </p>
            </div>
          </li>
        ))}
      </ul>
      <button className="mt-5 px-6 py-2.5 rounded-full bg-[#6425fe] text-white flex items-center justify-center gap-2">
        <span>Manage</span>
        <img
          src="https://placeholder.pics/svg/20x20"
          alt="Manage icon"
          className="w-5 h-5"
        />
      </button>
    </div>
  );
};

export default Devices;
