import React from 'react';
import NavBar from '../components/NavBar';
import { Outlet } from 'react-router-dom';
import ProxyProvider from '../ProxyProvider';
import ProtocolProvider from '../ProtocolProvider';

export default function Root() {
  return (
    <>
      <ProxyProvider>
        <ProtocolProvider>
          <NavBar />
          <Outlet />
        </ProtocolProvider>
      </ProxyProvider>
    </>
  );
}
