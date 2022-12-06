import React from 'react';
import Client from '../components/Client';
import './app.css';
import { Tabs, TabButton, TabPanel } from 'agnostic-react';
import { usePlayers } from '../ProxyProvider';
import Editor from '../components/Editor';

export default function App() {
  const players = usePlayers();

  const tabButtons = players.map((playerId) => (
    <TabButton key={playerId} id={playerId}>
      {`Player ${playerId}`}
    </TabButton>
  ));

  const tabPanels = players.map((playerId) => (
    <TabPanel key={playerId} id={playerId}>
      <Client playerId={playerId} />
    </TabPanel>
  ));

  return (
    <section id="app" style={{ margin: '10px' }}>
      <div className="players">
        {players.length ? (
          <Tabs tabButtons={tabButtons} tabPanels={tabPanels} />
        ) : (
          <p>Please connect a client to the proxy</p>
        )}
      </div>
      <Editor />
    </section>
  );
}
