import React from "react";
const SensorMap = () => {
  return (
    <div className="w-[850px] h-[533] flex flex-row p-6 gap-5 bg-white rounded-lg ">
      <div className="w-[850px] h-[533]">
        <h2
          className="font-poppins text-[15px] leading-[22.5px]
                     text-black mb-6 font-medium"
        >
          Your sensor map
        </h2>
        <img
          src="https://placeholder.pics/svg/680x432"
          alt="Map with sensor points"
          className="w-[781px] h-[432px]"
        />
      </div>
    </div>
  );
};

export default SensorMap;
