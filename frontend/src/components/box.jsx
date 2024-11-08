import React from "react";

const Box = ({ title, value, backgroundColor }) => {
  return (
    <div
      className="rounded-lg p-4 w-[203px] h-[115px] m-2 text-left"
      style={{ backgroundColor: backgroundColor }}
    >
      <h3 className="my-2 text-[17.5px] font-normal text-black font-poppins">
        {title}
      </h3>
      <p className=" text-sm text-BoxTextColor2 font-poppins font-normal">
        Current Value
      </p>
      <h4 className="text-[15px] text-BoxTextColor font-poppins font-medium">
        {value}
      </h4>
    </div>
  );
};

export default Box;
