import React, { useState, useEffect, useCallback } from 'react';
import MonacoEditor from '@monaco-editor/react';
import './Editor.css';
import { useProtocol, useReloadModule } from '../../ProtocolProvider';
import useDebounce from '../../hooks/useDebounce';

const darkThemeMq = window.matchMedia('(prefers-color-scheme: dark)');

export default function Editor() {
  const { protocol, loading } = useProtocol();
  const reloadModule = useReloadModule();
  const [code, setCode] = useState('');

  const update = useCallback((value) => {
    setCode(value);
    reloadModule(value);
  });

  useEffect(() => {
    reloadModule(protocol);
  }, []);

  useEffect(() => {
    setCode(loading ? 'Loading...' : protocol);
  }, [loading]);

  return (
    <div className="editor">
      <MonacoEditor
        height="100%"
        language="plaintext"
        defaultValue={code}
        theme={darkThemeMq.matches ? 'vs-dark' : 'vs-light'}
        onChange={useDebounce(update, 1000)}
      />
    </div>
  );
}
