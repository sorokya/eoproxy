import React, { useMemo } from 'react';
import PropTypes from 'prop-types';
import './Packet.css';
import getPacketId, { getActionName, getFamilyName} from '../../utils/getPacketId';
import { StreamReader } from '../../utils/streamReader';

Packet.propTypes = {
  packet: PropTypes.shape({
    from: PropTypes.oneOf(['Client', 'Server']).isRequired,
    buf: PropTypes.arrayOf(PropTypes.number).isRequired
  }).isRequired,
};

export default function Packet({ packet }) {
  const { from, buf } = packet;

  const action = useMemo(() => buf[0], [buf]);
  const family = useMemo(() => buf[1], [buf]);

  const packetJson = useMemo(() => {
    try {
      const className = `${from}${getFamilyName({family})}${getActionName({action})}`;
      const packet = new window[className]();
      const reader = new StreamReader(buf);

      if (from === 'Client' && family !== 255) {
        reader.getChar(); // sequence
      }

      // eslint-disable-next-line react/prop-types
      packet.deserialize(reader);

      // eslint-disable-next-line react/prop-types
      if (packet.password) {
        // eslint-disable-next-line react/prop-types
        packet.password = '********';
      }

      return JSON.stringify(packet, null, 2);
    } catch (e) {
      console.error(e);
      return null;
    }
  }, [buf]);

  return (
    <div className="packet" data-from={from}>
      <div className="header">
        <div className="from">
          {from === 'Client' ? 'C' : 'S'} ➡ {from === 'Client' ? 'S' : 'C'}
        </div>
        <div className="id">{getPacketId({ action, family })}</div>
      </div>

      {!packetJson && <pre className="data">{buf.join(' ')}</pre>}
      {!!packetJson && <pre className="data-parsed">{packetJson}</pre>}
      {!packetJson && <img src="/invalid.png" alt="Invalid protocol" />}
      {!!packetJson && <img src="/valid.png" alt="Valid protocol" />}
    </div>
  );
}
