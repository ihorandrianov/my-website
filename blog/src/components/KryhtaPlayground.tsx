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
      const ranges: { from: number; to: number; mark: Decoration }[] = [];
      const text = view.state.doc.toString();

      for (const match of text.matchAll(effectKeywords)) {
        ranges.push({
          from: match.index!,
          to: match.index! + match[0].length,
          mark: Decoration.mark({ class: 'cm-effect-keyword' }),
        });
      }

      for (const match of text.matchAll(effectNames)) {
        ranges.push({
          from: match.index!,
          to: match.index! + match[0].length,
          mark: Decoration.mark({ class: 'cm-effect-name' }),
        });
      }

      for (const match of text.matchAll(handlerArrow)) {
        ranges.push({
          from: match.index!,
          to: match.index! + match[0].length,
          mark: Decoration.mark({ class: 'cm-handler-arrow' }),
        });
      }

      ranges.sort((a, b) => a.from - b.from);
      return Decoration.set(ranges.map(r => r.mark.range(r.from, r.to)));
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

const EXAMPLES: Record<string, string> = {
  default: [
    '// Ask for values and sum them',
    'let result = handle {',
    '  let x = perform Ask!("first");',
    '  let y = perform Ask!("second");',
    '  x + y',
    '} with {',
    '  Ask!(label, resume) -> {',
    '    perform Print!("asking for", label);',
    '    resume(21)',
    '  },',
    '  return(x) -> x',
    '};',
    '',
    'perform Print!("sum:", result);',
    'result',
  ].join('\n'),

  generator: [
    '// Generator with Yield! effect',
    'function numbers(n) {',
    '  let i = 0;',
    '  while (i < n) {',
    '    perform Yield!(i);',
    '    i = i + 1;',
    '  }',
    '}',
    '',
    '// Consumer handles yields',
    'handle {',
    '  numbers(5)',
    '} with {',
    '  Yield!(value, resume) -> {',
    '    perform Print!("yielded:", value);',
    '    resume(undefined)',
    '  }',
    '}',
  ].join('\n'),

  state: [
    '// Mutable cell using effects',
    'let result = handle {',
    '  perform Set!(10);',
    '  let x = perform Get!();',
    '  perform Set!(x * 2);',
    '  perform Get!()',
    '} with {',
    '  Set!(v, resume) -> (s) => resume(undefined)(v),',
    '  Get!(resume) -> (s) => resume(s)(s),',
    '  return(x) -> (s) => x',
    '}(0);',
    '',
    'perform Print!("Final value:", result);',
    'result',
  ].join('\n'),

  exit: [
    '// Early termination - stop after finding 3',
    'handle {',
    '  let i = 0;',
    '  while (i < 100) {',
    '    if (i === 3) {',
    '      perform Exit!(i)',
    '    }',
    '    perform Print!("checking", i);',
    '    i = i + 1;',
    '  }',
    '  "not found"',
    '} with {',
    '  Exit!(value, resume) -> {',
    '    perform Print!("found it!");',
    '    value',
    '  }',
    '}',
  ].join('\n'),

  handler: [
    '// Handler as a value',
    'let counter = handler {',
    '  Inc!(resume) -> (n) => resume(undefined)(n + 1),',
    '  Dec!(resume) -> (n) => resume(undefined)(n - 1),',
    '  Get!(resume) -> (n) => resume(n)(n),',
    '  return(x) -> (n) => x',
    '};',
    '',
    '// Reuse the same handler',
    'let result1 = handle {',
    '  perform Inc!();',
    '  perform Inc!();',
    '  perform Get!()',
    '} with counter;',
    '',
    'let result2 = handle {',
    '  perform Inc!();',
    '  perform Dec!();',
    '  perform Dec!();',
    '  perform Get!()',
    '} with counter;',
    '',
    'perform Print!("counter1:", result1(0));',
    'perform Print!("counter2:", result2(0));',
    '"done"',
  ].join('\n'),
};

export default function KryhtaPlayground({ example = 'default' }: { example?: string }) {
  const [code, setCode] = useState(EXAMPLES[example] || EXAMPLES.default);
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
