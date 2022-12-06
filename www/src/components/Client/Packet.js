import React, { useMemo } from 'react';
import PropTypes from 'prop-types';
import './Packet.css';
import getPacketId from '../../utils/getPacketId';

Packet.propTypes = {
  packet: PropTypes.shape({
    from: PropTypes.oneOf(['Client', 'Server']).isRequired,
    buf: PropTypes.arrayOf(PropTypes.number).isRequired
  }).isRequired,
  view: PropTypes.oneOf(['raw', 'parsed']).isRequired
};

export default function Packet({ packet, view }) {
  const { from, buf } = packet;

  const action = useMemo(() => buf[0], [buf]);
  const family = useMemo(() => buf[1], [buf]);

  const packetJson = useMemo(() => {
    return 'TODO';
  }, [buf, view]);

  return (
    <div className="packet" data-from={from}>
      <div className="header">
        <div className="from">
          {from === 'Client' ? 'C' : 'S'} ➡ {from === 'Client' ? 'S' : 'C'}
        </div>
        <div className="id">{getPacketId({ action, family })}</div>
      </div>

      {view === 'raw' && <pre className="data">{buf.join(' ')}</pre>}
      {view === 'parsed' && <pre className="data-parsed">{packetJson}</pre>}
    </div>
  );
}
