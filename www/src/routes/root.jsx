import React from 'react';
import NavBar from '../components/NavBar';
import { Outlet } from 'react-router-dom';
import ProxyProvider from '../ProxyProvider';

export default function Root() {
  return (
    <>
      <ProxyProvider>
        <NavBar />
        <Outlet />
      </ProxyProvider>
    </>
  );
}
