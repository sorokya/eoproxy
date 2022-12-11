import React from 'react';
import PropTypes from 'prop-types';
import './PacketList.css';
import Packet from './Packet';
import { Scrollbars } from 'react-custom-scrollbars';

PacketList.propTypes = {
  packets: PropTypes.arrayOf(
    PropTypes.shape({
      id: PropTypes.number.isRequired,
      name: PropTypes.string.isRequired
    })
  ).isRequired
};

export default function PacketList({ packets }) {
  return (
    <div className="packetList">
      <Scrollbars style={{ width: 420, height: '85vh' }}>
        <ul>
          {packets
            .slice(0)
            .reverse()
            .slice(0, 20)
            .map((packet, i) => (
              <li key={i}>
                <Packet packet={packet} />
              </li>
            ))}
        </ul>
      </Scrollbars>
    </div>
  );
}
