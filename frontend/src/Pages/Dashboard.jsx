import React from "react";
import SidebarMenu from "../components/sidebar";
import Navbar from "../components/Navbar";
import Box from "../components/box";
import SensorMap from "../components/SensorMap";
import AnalyticsPanel from "../components/AnalyticsPanel";
import Devices from "../components/Devices";

const ClusterStatus = () => {
  return (
    <div className="p-3 grid flex-row gap-5 bg-white rounded-lg">
      <div className="flex-row font-poppins text-titleColor font-medium">
        My cluster
      </div>
      <div className="flex">
        <Box title="Memory" value="10%" backgroundColor="#a6f7e2" />
        <Box title="CPU" value="50%" backgroundColor="#d5c4ff" />
        <Box title="Node Status" value="Online" backgroundColor="#ffe5a5" />
        <Box title="Deployment Status" value="70%" backgroundColor="#c7ffa5" />
      </div>
    </div>
  );
};

const Dashboard = () => {
  return (
    <>

      <div className="flex flex-row bg-gray-100 min-h-screen">
        {/* Sidebar */}
        <SidebarMenu />

        {/* Main Content */}
        <div className="flex flex-col gap-4 p-6 flex-1">
        <Navbar />
          <ClusterStatus />

          {/* Grid layout: SensorMap and Devices side by side, AnalyticsPanel below */}
          <div className="grid grid-cols-2 md:grid-cols-1 gap-8">
            {/* First Row: SensorMap and Devices side by side with increased gap */}
            <div className="flex flex-col gap-4">
              <SensorMap />
            </div>
            <div className="flex flex-col gap-4">
              <Devices />
            </div>

            {/* Second Row: AnalyticsPanel */}
            <div className="flex flex-col gap-4 col-span-2">
              <AnalyticsPanel />
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

export default Dashboard;
