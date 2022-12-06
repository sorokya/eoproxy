/* eslint-disable react/prop-types */
import React from 'react';
import './PacketInspector.css';
import Packet from './Packet';

// eslint-disable-next-line react/prop-types
export default function PacketInspector({ packet }) {
  return (
    <div className="packetInspector">
      {packet ? (
        <Packet packet={packet} view="parsed" />
      ) : (
        <p>Please select a packet from the list</p>
      )}
    </div>
  );
}
