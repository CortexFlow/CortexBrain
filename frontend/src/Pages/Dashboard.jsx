import React from 'react';
import '../assets/css/dashboard.css';
import SidebarMenu from '../components/sidebar';
import Box from '../components/box';
import SensorMap from '../components/SensorMap';
import AnalyticsPanel from '../components/AnalyticsPanel';
// Importing components


const ClusterStatus = () => {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '12px' }}>
      <Box title="Memory" value="10%" backgroundColor="#a6f7e2" />
      <Box title="CPU" value="50%" backgroundColor="#d5c4ff" />
      <Box title="Node Status" value="Online" backgroundColor="#ffe5a5" />
      <Box title="Deployment Status" value="70%" backgroundColor="#c7ffa5" />
    </div>
  );
};

const Dashboard = () => {
  return (
    <div style={{ display: 'flex', flexDirection: 'row', backgroundColor: '#f5f5f5' }}>
      <SidebarMenu />
      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', padding: '24px', flex: 1 }}>
        <ClusterStatus />
        <div style={{ display: 'flex', gap: '16px' }}>
          <SensorMap />
          <AnalyticsPanel />
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
