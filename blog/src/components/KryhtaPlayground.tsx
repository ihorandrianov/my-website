import { useState, useRef, useEffect } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { javascript } from '@codemirror/lang-javascript';
import { ViewPlugin, Decoration, EditorView } from '@codemirror/view';
import type { DecorationSet } from '@codemirror/view';

const effectKeywords = /\b(perform|handle|with|resume)\b/g;
const effectNames = /\b[A-Z][a-zA-Z]*!/g;
const handlerArrow = /->/g;

const kryhtaHighlighter = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = this.buildDecorations(view);
    }
    update(update: { docChanged: boolean; view: EditorView }) {
      if (update.docChanged) {
        this.decorations = this.buildDecorations(update.view);
      }
    }
    buildDecorations(view: EditorView) {
      const decorations: { from: number; to: number; decoration: Decoration }[] = [];
      const text = view.state.doc.toString();

      for (const match of text.matchAll(effectKeywords)) {
        decorations.push({
          from: match.index!,
          to: match.index! + match[0].length,
          decoration: Decoration.mark({ class: 'cm-effect-keyword' }),
        });
      }

      for (const match of text.matchAll(effectNames)) {
        decorations.push({
          from: match.index!,
          to: match.index! + match[0].length,
          decoration: Decoration.mark({ class: 'cm-effect-name' }),
        });
      }

      for (const match of text.matchAll(handlerArrow)) {
        decorations.push({
          from: match.index!,
          to: match.index! + match[0].length,
          decoration: Decoration.mark({ class: 'cm-handler-arrow' }),
        });
      }

      return Decoration.set(decorations.sort((a, b) => a.from - b.from));
    }
  },
  { decorations: (v) => v.decorations }
);

const kryhtaTheme = EditorView.baseTheme({
  '.cm-effect-keyword': { color: '#c586c0', fontWeight: 'bold' },
  '.cm-effect-name': { color: '#4ec9b0' },
  '.cm-handler-arrow': { color: '#d7ba7d' },
});

type KryhtaModule = {
  evaluate: (source: string) => string;
};

const DEFAULT_CODE = `// Algebraic effects in action!
let result = handle {
  let x = perform Get!();
  perform Put!(x + 1);
  perform Get!()
} with {
  Get!(resume) -> resume(42),
  Put!(value, resume) -> resume(value * 2),
  return(x) -> x
};

perform Print!("Result:", result);
result`;

export default function KryhtaPlayground({ initialCode }: { initialCode?: string }) {
  const [code, setCode] = useState(initialCode || DEFAULT_CODE);
  const [output, setOutput] = useState<string[]>([]);
  const [result, setResult] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const wasmRef = useRef<KryhtaModule | null>(null);

  useEffect(() => {
    async function loadWasm() {
      try {
        const module = await import('../wasm/kryhta.js');
        await module.default();
        wasmRef.current = module;
        setLoading(false);
      } catch (e) {
        setError(`Failed to load WASM: ${e}`);
        setLoading(false);
      }
    }
    loadWasm();
  }, []);

  const runCode = () => {
    if (!wasmRef.current) return;

    const captured: string[] = [];
    const originalLog = console.log;
    console.log = (...args: unknown[]) => {
      captured.push(args.map(String).join(' '));
    };

    try {
      const res = wasmRef.current.evaluate(code);
      setResult(res);
      setOutput(captured);
      setError(null);
    } catch (e) {
      setError(String(e));
      setOutput(captured);
    } finally {
      console.log = originalLog;
    }
  };

  return (
    <div style={styles.container}>
      <CodeMirror
        value={code}
        height="250px"
        extensions={[javascript(), kryhtaHighlighter, kryhtaTheme]}
        onChange={setCode}
        theme="dark"
        basicSetup={{
          lineNumbers: true,
          foldGutter: false,
          highlightActiveLineGutter: false,
        }}
      />
      <div style={styles.controls}>
        <button onClick={runCode} disabled={loading} style={styles.button}>
          {loading ? 'Loading...' : 'Run'}
        </button>
      </div>
      <div style={styles.output}>
        <div style={styles.outputHeader}>Output</div>
        {error && <div style={styles.error}>{error}</div>}
        {output.map((line, i) => (
          <div key={i} style={styles.outputLine}>{line}</div>
        ))}
        {result && !error && (
          <div style={styles.result}>→ {result}</div>
        )}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    border: '1px solid #333',
    borderRadius: '8px',
    overflow: 'hidden',
    fontFamily: 'monospace',
    margin: '1.5rem 0',
  },
  controls: {
    backgroundColor: '#252525',
    padding: '0.5rem 1rem',
    borderTop: '1px solid #333',
    borderBottom: '1px solid #333',
  },
  button: {
    backgroundColor: '#4a9eff',
    color: 'white',
    border: 'none',
    padding: '0.5rem 1.5rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontFamily: 'monospace',
    fontSize: '14px',
  },
  output: {
    backgroundColor: '#0d0d0d',
    padding: '1rem',
    minHeight: '80px',
  },
  outputHeader: {
    color: '#666',
    fontSize: '12px',
    marginBottom: '0.5rem',
    textTransform: 'uppercase',
  },
  outputLine: {
    color: '#a0a0a0',
    fontSize: '14px',
    lineHeight: '1.4',
  },
  result: {
    color: '#4a9eff',
    fontSize: '14px',
    marginTop: '0.5rem',
  },
  error: {
    color: '#ff6b6b',
    fontSize: '14px',
  },
};
