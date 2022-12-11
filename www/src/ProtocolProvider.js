import React, { createContext, useState, useEffect, useCallback } from 'react';
import {parse} from 'eo-protocol-parser';

const ProtocolContext = createContext({});

function getProtocolValue() {
  // fetch from localStorage
  const protocol = localStorage.getItem('protocol');
  return new Promise((resolve) => {
    if (!protocol) {
      // fetch from github
      fetch(
        'https://raw.githubusercontent.com/sorokya/eo_protocol/v28/eo.txt'
      ).then((response) => {
        response.text().then((text) => {
          localStorage.setItem('protocol', text.trim());
          resolve(text.trim());
        })
      });
    } else {
      resolve(protocol);
    }
  });
}

// eslint-disable-next-line react/prop-types
export default function ProtocolProvider({ children }) {
  const [protocol, setProtocol] = useState('');
  const [loading, setLoading] = useState(undefined);

  useEffect(() => {
    if (typeof loading === 'undefined') {
      setLoading(true);
      getProtocolValue().then((protocolValue) => {
        setProtocol(protocolValue);
        setLoading(false);
      });
    }
  }, []);

  const reloadModule = useCallback((source) => {
    const {protocol: protocolSource} = parse({
      protocolSource: source,
      language: 'javascript',
    });

    eval?.(protocolSource);
  }, []);

  return (
    <ProtocolContext.Provider value={{ protocol, setProtocol, loading, reloadModule }}>
      {children}
    </ProtocolContext.Provider>
  );
}

export function useProtocol() {
  const context = React.useContext(ProtocolContext);

  if (context === undefined) {
    throw new Error('useProxy must be used within a ProtocolProvider');
  }

  return {
    protocol: context.protocol,
    loading: context.loading,
  };
}

export function useSetProtocol() {
  const context = React.useContext(ProtocolContext);

  if (context === undefined) {
    throw new Error('useProxy must be used within a ProtocolProvider');
  }

  return context.setProtocol;
}

export function useReloadModule() {
  const context = React.useContext(ProtocolContext);

  if (context === undefined) {
    throw new Error('useReloadModule must be used within a ProtocolProvider');
  }

  return context.reloadModule;
}
