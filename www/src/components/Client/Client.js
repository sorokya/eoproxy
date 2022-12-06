import React, { useState } from 'react';
import PropTypes from 'prop-types';
import './Client.css';
import PacketList from './PacketList';
import PacketInspector from './PacketInspector';
import { usePackets } from '../../ProxyProvider';

Client.propTypes = {
  playerId: PropTypes.number.isRequired
};

export default function Client({ playerId }) {
  const packets = usePackets(playerId);

  const [selectedPacketIndex] = useState(0);

  return (
    <section className="client">
      <PacketList packets={packets} />
      <PacketInspector
        packet={
          selectedPacketIndex > -1 ? packets[selectedPacketIndex] : undefined
        }
      />
    </section>
  );
}
