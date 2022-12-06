import React, { useState } from 'react';
import CodeEditor from '@uiw/react-textarea-code-editor';
import './Editor.css';

export default function Editor() {
  const [code, setCode] = useState('');
  return (
    <div className="editor">
      <CodeEditor
        value={code}
        language="eo_protocol"
        placeholder="Please enter protocol code"
        onChange={(evn) => setCode(evn.target.value)}
        padding={15}
        style={{
          fontSize: 12,
          backgroundColor: '#1B272C',
          height: '100%',
          fontFamily:
            'ui-monospace,SFMono-Regular,SF Mono,Consolas,Liberation Mono,Menlo,monospace'
        }}
      />
    </div>
  );
}
