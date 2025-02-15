import React from "react";

const AnalyticsPanel = () => {
  return (
    <div className="relative w-[681px] h-[504px] p-6 bg-white rounded-lg shadow-md">
      <h2 className="font-normal text-[15px] leading-[22.5px] text-black mb-5 ml-3">
        Analytics
      </h2>
      <div
        className="absolute top-[60px] left-3 w-[621.16px] h-[378.44px] bg-center bg-cover"
        style={{
          backgroundImage: "url(https://placeholder.pics/svg/621x378)",
        }}
      ></div>
      <div className="absolute bottom-[30px] w-full flex justify-between px-14">
        <span className="font-semibold text-[12px] text-[#33323a]">2014</span>
        <span className="font-semibold text-[12px] text-[#33323a]">2016</span>
        <span className="font-semibold text-[12px] text-[#33323a]">2018</span>
        <span className="font-semibold text-[12px] text-[#33323a]">2020</span>
        <span className="font-semibold text-[12px] text-[#33323a]">2022</span>
      </div>
    </div>
  );
};

export default AnalyticsPanel;
