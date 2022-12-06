import React, { createContext, useState, useEffect, useMemo } from 'react';
import useWebSocket, { ReadyState } from 'react-use-websocket';

const ProxyContext = createContext({});

// eslint-disable-next-line react/prop-types
export default function ProxyProvider({ children }) {
  const [packets, setPackets] = useState([]);
  const [players, setPlayers] = useState([]);
  const { lastMessage, readyState } = useWebSocket('ws://localhost:9001');

  useEffect(() => {
    if (lastMessage !== null) {
      const command = JSON.parse(lastMessage.data);

      console.log(lastMessage.data);

      if (command === 'AddPlayer' && !players.includes(0)) {
        setPlayers((prev) => prev.concat(0));
        return;
      }

      const { Packet, SetPlayerId, RemovePlayer } = command;

      if (SetPlayerId) {
        setPlayers((prev) => {
          prev[prev.indexOf(0)] = SetPlayerId;
          return prev;
        });
        setPackets((prev) =>
          prev.map((p) => ({
            ...p,
            player_id: p.player_id === 0 ? SetPlayerId : p.player_id
          }))
        );
        return;
      }

      if (Packet) {
        setPackets((prev) => prev.concat(Packet));
        return;
      }

      if (RemovePlayer) {
        setPlayers((prev) => prev.filter((p) => p !== RemovePlayer));
        return;
      }
    }
  }, [lastMessage, setPackets, setPlayers]);

  const connectionStatus = useMemo(
    () =>
      ({
        [ReadyState.CONNECTING]: 'Connecting',
        [ReadyState.OPEN]: 'Open',
        [ReadyState.CLOSING]: 'Closing',
        [ReadyState.CLOSED]: 'Closed',
        [ReadyState.UNINSTANTIATED]: 'Uninstantiated'
      }[readyState]),
    [readyState]
  );

  return (
    <ProxyContext.Provider value={{ packets, connectionStatus, players }}>
      {children}
    </ProxyContext.Provider>
  );
}

export function useConnectionStatus() {
  const context = React.useContext(ProxyContext);

  if (context === undefined) {
    throw new Error('useProxy must be used within a ProxyProvider');
  }

  return context.connectionStatus;
}

export function usePackets(playerId) {
  const context = React.useContext(ProxyContext);

  if (context === undefined) {
    throw new Error('usePackets must be used within a ProxyProvider');
  }

  return context.packets.filter((m) => m.player_id === playerId);
}

export function usePlayers() {
  const context = React.useContext(ProxyContext);

  if (context === undefined) {
    throw new Error('usePlayers must be used within a ProxyProvider');
  }

  return context.players;
}
