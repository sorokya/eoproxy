import React from 'react';
import { Link } from 'react-router-dom';
import { Header, HeaderNav, HeaderNavItem, Icon } from 'agnostic-react';
import { ReactComponent as ConnectedIcon } from '../icons/connected.svg';
import { ReactComponent as DisconnectedIcon } from '../icons/disconnected.svg';
import './NavBar.css';
import { useConnectionStatus } from '../ProxyProvider';

export default function NavBar() {
  const connectionStatus = useConnectionStatus();
  // const icon = React.useMemo(
  //   () =>
  //     connectionStatus === 'Open' ? (
  //       <Icon size={12} type="info">
  //         <ConnectedIcon />
  //       </Icon>
  //     ) : (
  //       <Icon size={12} type="info">
  //         <DisconnectedIcon />
  //       </Icon>
  //     ),
  //   [connectionStatus]
  // );

  return (
    <Header>
      <>
        <span className="flex-fill">
          <Link to="/">Endless Protocol Editor</Link>
        </span>
        <HeaderNav>
          <HeaderNavItem>
            <Link to="/">Editor</Link>
          </HeaderNavItem>
          <HeaderNavItem>
            <Link to="/settings">Settings</Link>
          </HeaderNavItem>
          <HeaderNavItem>
            <span
              className="connectionStatus"
              title={connectionStatus === 'Open' ? 'Connected' : 'Disconnected'}
              style={{
                display: connectionStatus === 'Open' ? 'block' : 'none'
              }}>
              <Icon size={12} type="info">
                <ConnectedIcon />
              </Icon>
            </span>
            <span
              className="connectionStatus"
              title={connectionStatus === 'Open' ? 'Connected' : 'Disconnected'}
              style={{
                display: connectionStatus === 'Open' ? 'none' : 'block'
              }}>
              <Icon size={12} type="info">
                <DisconnectedIcon />
              </Icon>
            </span>
          </HeaderNavItem>
        </HeaderNav>
      </>
    </Header>
  );
}
